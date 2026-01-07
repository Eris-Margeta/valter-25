use crate::config::Config;
use anyhow::{Context, Result};
use rusqlite::{params, types::Value as SqlValue, Connection};
use serde_json::{Map, Value as JsonValue};
use std::collections::{HashMap, HashSet};
use std::sync::Mutex;
use strsim::levenshtein;
use tracing::{info, warn};
use uuid::Uuid;

#[allow(dead_code)]
pub struct SqliteManager {
    conn: Mutex<Connection>,
}

#[allow(dead_code)]
#[derive(Debug)]
pub enum EntityStatus {
    Found(String),
    Pending(()),
    Ambiguous(String, Vec<String>),
}

impl SqliteManager {
    pub fn new(path: &str) -> Result<Self> {
        let conn = Connection::open(path).context("Failed to open SQLite DB")?;

        // PRAGMA journal_mode=WAL is excellent for concurrency
        let _: String = conn
            .query_row("PRAGMA journal_mode=WAL", [], |row| row.get(0))
            .unwrap_or("delete".to_string());

        conn.execute("PRAGMA synchronous=NORMAL", [])?;

        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    pub fn init_schema(&self, config: &Config) -> Result<()> {
        let conn = self.conn.lock().unwrap();

        // 0. SYSTEM METADATA
        conn.execute(
            "CREATE TABLE IF NOT EXISTS _valter_system (key TEXT PRIMARY KEY, value TEXT)",
            [],
        )?;
        conn.execute(
            "INSERT OR REPLACE INTO _valter_system (key, value) VALUES ('schema_version', '1')",
            [],
        )?;

        // 1. CLOUDS (Dynamic Migration)
        for cloud_def in &config.clouds {
            self.ensure_table(&conn, &cloud_def.name, &cloud_def.fields, false)?;
        }

        // 2. ISLANDS (Dynamic Migration)
        for island_def in &config.islands {
            let mut virtual_fields = Vec::new();

            for rel in &island_def.relations {
                virtual_fields.push(crate::config::CloudField {
                    key: rel.field.clone(),
                    field_type: "string".to_string(),
                    required: false,
                    options: None,
                });
            }
            for agg in &island_def.aggregations {
                virtual_fields.push(crate::config::CloudField {
                    key: agg.name.clone(),
                    field_type: "number".to_string(),
                    required: false,
                    options: None,
                });
            }

            self.ensure_table(&conn, &island_def.name, &virtual_fields, true)?;
        }

        // 3. PENDING ACTIONS (Fixed Schema)
        let pending_query = "
            CREATE TABLE IF NOT EXISTS pending_actions (
                id TEXT PRIMARY KEY,
                type TEXT NOT NULL,
                target_table TEXT NOT NULL,
                key_field TEXT NOT NULL,
                value TEXT NOT NULL,
                context TEXT,
                suggestions TEXT,
                status TEXT DEFAULT 'Pending',
                created_at TEXT
            )
        ";
        conn.execute(pending_query, [])?;

        Ok(())
    }

    /// PURGE ISLANDS: Briše sve zapise iz tablice otoka.
    /// Ovo koristimo kod Rescana da osiguramo da je SQL baza 1-na-1 sa Filesystemom.
    pub fn purge_islands(&self, table_name: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        // Provjera postoji li tablica prije brisanja
        let exists: bool = conn
            .query_row(
                "SELECT count(*) FROM sqlite_master WHERE type='table' AND name=?",
                params![table_name],
                |row| row.get(0),
            )
            .unwrap_or(0)
            > 0;

        if exists {
            conn.execute(&format!("DELETE FROM {}", table_name), [])?;
            info!("🧹 Purged all data from Island table: {}", table_name);
        }
        Ok(())
    }

    /// RESET PENDING: Briše sve akcije kako bi se ponovno generirale kod skeniranja.
    pub fn reset_pending_actions(&self) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM pending_actions", [])?;
        info!("🧹 Purged all Pending Actions (Notifications reset).");
        Ok(())
    }

    fn ensure_table(
        &self,
        conn: &Connection,
        table_name: &str,
        fields: &[crate::config::CloudField],
        is_island: bool,
    ) -> Result<()> {
        let mut expected_cols = HashMap::new();
        expected_cols.insert("id".to_string(), "TEXT PRIMARY KEY".to_string());

        if is_island {
            expected_cols.insert("name".to_string(), "TEXT".to_string());
            expected_cols.insert("path".to_string(), "TEXT".to_string());
            expected_cols.insert("status".to_string(), "TEXT".to_string());
            expected_cols.insert("updated_at".to_string(), "TEXT".to_string());
        }

        for field in fields {
            let sql_type = match field.field_type.as_str() {
                "number" => "REAL",
                "boolean" => "INTEGER",
                _ => "TEXT",
            };
            expected_cols.insert(field.key.clone(), sql_type.to_string());
        }

        let table_exists: bool = conn
            .query_row(
                "SELECT count(*) FROM sqlite_master WHERE type='table' AND name=?",
                params![table_name],
                |row| row.get(0),
            )
            .unwrap_or(0)
            > 0;

        if !table_exists {
            let mut cols_def = Vec::new();
            for (col, type_def) in &expected_cols {
                cols_def.push(format!("{} {}", col, type_def));
            }
            let query = format!("CREATE TABLE {} ({})", table_name, cols_def.join(", "));
            info!("Database: Creating table '{}'", table_name);
            conn.execute(&query, [])?;
        } else {
            let mut stmt = conn.prepare(&format!("PRAGMA table_info({})", table_name))?;
            let existing_cols: HashSet<String> = stmt
                .query_map([], |row| row.get::<_, String>(1))?
                .filter_map(|r| r.ok())
                .collect();

            for (col, type_def) in &expected_cols {
                if !existing_cols.contains(col) {
                    let clean_type = type_def.replace("PRIMARY KEY", "");
                    let query = format!(
                        "ALTER TABLE {} ADD COLUMN {} {}",
                        table_name, col, clean_type
                    );
                    info!(
                        "Database: Migrating '{}' -> Adding column '{}'",
                        table_name, col
                    );
                    if let Err(e) = conn.execute(&query, []) {
                        warn!("Failed to migrate column {}.{}: {}", table_name, col, e);
                    }
                }
            }
        }
        Ok(())
    }

    pub fn check_or_create_pending(
        &self,
        table: &str,
        key_field: &str,
        value: &str,
        context_info: &str,
    ) -> Result<EntityStatus> {
        let conn = self.conn.lock().unwrap();

        // 1. Check Exact Match
        let query_exact = format!("SELECT id FROM {} WHERE {} = ?", table, key_field);
        {
            let mut stmt = conn.prepare(&query_exact)?;
            let mut rows = stmt.query(params![value])?;
            if let Some(row) = rows.next()? {
                return Ok(EntityStatus::Found(row.get(0)?));
            }
        }

        // 2. Check if already Pending (Ignore status for now, we just check existence)
        // Zapravo, želimo vratiti pending samo ako je 'Pending'. Ako je 'Rejected', želimo ga ponovno prikazati?
        // User je tražio "RESCAN" koji resetira stvari.
        // Ovdje provjeravamo samo 'Pending' status.
        let query_pending = "SELECT id FROM pending_actions WHERE target_table = ? AND value = ? AND status = 'Pending'";
        {
            let mut stmt = conn.prepare(query_pending)?;
            let mut rows = stmt.query(params![table, value])?;
            if let Some(_row) = rows.next()? {
                return Ok(EntityStatus::Pending(()));
            }
        }

        // 3. Create Suggestions & New Action
        let mut suggestions = Vec::new();
        if let Ok(mut stmt) = conn.prepare(&format!("SELECT {} FROM {}", key_field, table)) {
            let names_iter = stmt.query_map([], |row| row.get::<_, String>(0))?;
            for name_res in names_iter {
                if let Ok(existing_name) = name_res {
                    let dist = levenshtein(value, &existing_name);
                    if dist > 0 && dist <= 3 {
                        suggestions.push(existing_name);
                    }
                }
            }
        }

        let action_id = Uuid::new_v4().to_string();
        let suggestions_json = serde_json::to_string(&suggestions)?;
        let now = chrono::Local::now().to_rfc3339();

        let insert_sql = "
            INSERT INTO pending_actions (id, type, target_table, key_field, value, context, suggestions, status, created_at)
            VALUES (?, 'CreateEntity', ?, ?, ?, ?, ?, 'Pending', ?)
        ";
        conn.execute(
            insert_sql,
            params![
                action_id,
                table,
                key_field,
                value,
                context_info,
                suggestions_json,
                now
            ],
        )?;

        info!(
            "Safety Valve: '{}' not found in {}. Action Created.",
            value, table
        );

        if suggestions.is_empty() {
            Ok(EntityStatus::Pending(()))
        } else {
            Ok(EntityStatus::Ambiguous(action_id, suggestions))
        }
    }

    pub fn approve_pending_creation(&self, action_id: &str) -> Result<String> {
        let mut conn = self.conn.lock().unwrap();
        let tx = conn.transaction()?;

        let (table, key_field, value): (String, String, String) = {
            let mut stmt = tx.prepare(
                "SELECT target_table, key_field, value FROM pending_actions WHERE id = ?",
            )?;
            let mut rows = stmt.query(params![action_id])?;
            if let Some(row) = rows.next()? {
                (row.get(0)?, row.get(1)?, row.get(2)?)
            } else {
                anyhow::bail!("Action not found or already resolved.");
            }
        };

        let new_id = Uuid::new_v4().to_string();
        let query_insert = format!("INSERT INTO {} (id, {}) VALUES (?, ?)", table, key_field);

        tx.execute(&query_insert, params![new_id, value])?;
        tx.execute(
            "UPDATE pending_actions SET status = 'Resolved' WHERE id = ?",
            params![action_id],
        )?;

        tx.commit()?;
        info!("Approved & Created: {} (ID: {})", value, new_id);
        Ok(new_id)
    }

    pub fn reject_pending_action(&self, action_id: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE pending_actions SET status = 'Rejected' WHERE id = ?",
            params![action_id],
        )?;
        info!("Action Rejected: {}", action_id);
        Ok(())
    }

    pub fn upsert_island(
        &self,
        table: &str,
        name: &str,
        path: &str,
        relations: &HashMap<String, Option<String>>,
        aggregations: &HashMap<String, f64>,
    ) -> Result<()> {
        let conn = self.conn.lock().unwrap();

        // Check ID
        let query_select = format!("SELECT id FROM {} WHERE name = ?", table);
        let project_id: String = {
            let mut stmt = conn.prepare(&query_select)?;
            let mut rows = stmt.query(params![name])?;
            if let Some(row) = rows.next()? {
                row.get(0)?
            } else {
                Uuid::new_v4().to_string()
            }
        };

        // Delete old entry to ensure clean upsert (simpler than UPDATE for dynamic fields)
        conn.execute(
            &format!("DELETE FROM {} WHERE id = ?", table),
            params![project_id],
        )?;

        let mut final_cols = vec![
            "id".to_string(),
            "name".to_string(),
            "path".to_string(),
            "updated_at".to_string(),
        ];
        let now = chrono::Local::now().to_rfc3339();
        let mut final_vals = vec![
            format!("'{}'", project_id),
            format!("'{}'", name),
            format!("'{}'", path),
            format!("'{}'", now),
        ];

        for (k, v) in relations {
            final_cols.push(k.clone());
            match v {
                Some(uuid) => final_vals.push(format!("'{}'", uuid)),
                None => final_vals.push("NULL".to_string()),
            }
        }
        for (k, v) in aggregations {
            final_cols.push(k.clone());
            final_vals.push(format!("{}", v));
        }

        let query = format!(
            "INSERT INTO {} ({}) VALUES ({})",
            table,
            final_cols.join(", "),
            final_vals.join(", ")
        );
        conn.execute(&query, [])?;
        Ok(())
    }

    pub fn fetch_pending_actions(&self) -> Result<Vec<JsonValue>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = match conn.prepare("SELECT * FROM pending_actions WHERE status = 'Pending'")
        {
            Ok(s) => s,
            Err(_) => return Ok(vec![]),
        };

        let col_names: Vec<String> = stmt
            .column_names()
            .into_iter()
            .map(|s| s.to_string())
            .collect();
        let rows = stmt.query_map([], |row| {
            let mut map = Map::new();
            for i in 0..col_names.len() {
                let val: SqlValue = row.get(i)?;
                let json_val = match val {
                    SqlValue::Text(s) => {
                        if col_names[i] == "suggestions" {
                            serde_json::from_str(&s).unwrap_or(JsonValue::String(s))
                        } else {
                            JsonValue::String(s)
                        }
                    }
                    SqlValue::Null => JsonValue::Null,
                    _ => JsonValue::String("Unknown".to_string()),
                };
                map.insert(col_names[i].clone(), json_val);
            }
            Ok(JsonValue::Object(map))
        })?;

        let mut res = Vec::new();
        for r in rows {
            res.push(r?);
        }
        Ok(res)
    }

    pub fn fetch_all_dynamic(&self, table: &str) -> Result<Vec<JsonValue>> {
        let conn = self.conn.lock().unwrap();
        let query = format!("SELECT * FROM {}", table);

        let mut stmt = match conn.prepare(&query) {
            Ok(s) => s,
            Err(_) => return Ok(vec![]),
        };

        let col_names: Vec<String> = stmt
            .column_names()
            .into_iter()
            .map(|s| s.to_string())
            .collect();
        let rows = stmt.query_map([], |row| {
            let mut map = Map::new();
            for i in 0..col_names.len() {
                let val: SqlValue = row.get(i)?;
                let json_val = match val {
                    SqlValue::Null => JsonValue::Null,
                    SqlValue::Integer(i) => JsonValue::Number(i.into()),
                    SqlValue::Real(f) => {
                        if let Some(n) = serde_json::Number::from_f64(f) {
                            JsonValue::Number(n)
                        } else {
                            JsonValue::Null
                        }
                    }
                    SqlValue::Text(s) => JsonValue::String(s),
                    SqlValue::Blob(_) => JsonValue::String("<BINARY>".to_string()),
                };
                map.insert(col_names[i].clone(), json_val);
            }
            Ok(JsonValue::Object(map))
        })?;

        let mut results = Vec::new();
        for r in rows {
            results.push(r?);
        }
        Ok(results)
    }
}
