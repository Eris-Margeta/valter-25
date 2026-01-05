use rusqlite::{Connection, params, types::Value as SqlValue};
use uuid::Uuid;
use crate::config::Config;
use anyhow::{Result, Context};
use tracing::{info, debug, warn};
use std::sync::Mutex;
use std::collections::HashMap;
use serde_json::{Value as JsonValue, Map};
use strsim::levenshtein;

pub struct SqliteManager {
    conn: Mutex<Connection>,
}

#[derive(Debug)]
pub enum EntityStatus {
    Found(String),
    Pending(String),
    Ambiguous(String, Vec<String>),
}

impl SqliteManager {
    pub fn new(path: &str) -> Result<Self> {
        let conn = Connection::open(path).context("Failed to open SQLite DB")?;
        Ok(Self { conn: Mutex::new(conn) })
    }

    pub fn init_schema(&self, config: &Config) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        
        // 1. CLOUDS
        for cloud_def in &config.clouds {
            let table_name = &cloud_def.name;
            let mut columns = vec!["id TEXT PRIMARY KEY".to_string()];
            
            for field in &cloud_def.fields {
                let sql_type = match field.field_type.as_str() {
                    "number" => "REAL",
                    "boolean" => "INTEGER",
                    _ => "TEXT",
                };
                columns.push(format!("{} {}", field.key, sql_type));
            }

            let query = format!("CREATE TABLE IF NOT EXISTS {} ({})", table_name, columns.join(", "));
            if let Err(e) = conn.execute(&query, []) {
                warn!("Error creating Cloud table {}: {}", table_name, e);
            }
        }

        // 2. ISLANDS
        for island_def in &config.islands {
            let table_name = &island_def.name;
            let mut columns = vec![
                "id TEXT PRIMARY KEY".to_string(),
                "name TEXT".to_string(),
                "path TEXT".to_string(),
                "status TEXT".to_string(),
                "updated_at TEXT".to_string()
            ];

            for rel in &island_def.relations {
                columns.push(format!("{} TEXT", rel.field));
            }

            for agg in &island_def.aggregations {
                columns.push(format!("{} REAL", agg.name));
            }

            let query = format!("CREATE TABLE IF NOT EXISTS {} ({})", table_name, columns.join(", "));
            if let Err(e) = conn.execute(&query, []) {
                warn!("Error creating Island table {}: {}", table_name, e);
            }
        }

        // 3. PENDING ACTIONS
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

    pub fn check_or_create_pending(&self, table: &str, key_field: &str, value: &str, context_info: &str) -> Result<EntityStatus> {
        let conn = self.conn.lock().unwrap();

        let query_exact = format!("SELECT id FROM {} WHERE {} = ?", table, key_field);
        {
            let mut stmt = conn.prepare(&query_exact)?;
            let mut rows = stmt.query(params![value])?;
            if let Some(row) = rows.next()? {
                return Ok(EntityStatus::Found(row.get(0)?));
            }
        }

        let query_pending = "SELECT id FROM pending_actions WHERE target_table = ? AND value = ? AND status = 'Pending'";
        {
            let mut stmt = conn.prepare(query_pending)?;
            let mut rows = stmt.query(params![table, value])?;
            if let Some(row) = rows.next()? {
                return Ok(EntityStatus::Pending(row.get(0)?));
            }
        }

        let mut suggestions = Vec::new();
        let query_all = format!("SELECT {} FROM {}", key_field, table);
        {
            let mut stmt = conn.prepare(&query_all)?;
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
        conn.execute(insert_sql, params![action_id, table, key_field, value, context_info, suggestions_json, now])?;

        info!("Safety Valve: '{}' not found. Action Created. Suggestions: {:?}", value, suggestions);
        
        if suggestions.is_empty() {
            Ok(EntityStatus::Pending(action_id))
        } else {
            Ok(EntityStatus::Ambiguous(action_id, suggestions))
        }
    }

    pub fn approve_pending_creation(&self, action_id: &str) -> Result<String> {
        let mut conn = self.conn.lock().unwrap();
        let tx = conn.transaction()?;

        let (table, key_field, value): (String, String, String) = {
            let mut stmt = tx.prepare("SELECT target_table, key_field, value FROM pending_actions WHERE id = ?")?;
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
        tx.execute("UPDATE pending_actions SET status = 'Resolved' WHERE id = ?", params![action_id])?;

        tx.commit()?;
        info!("Approved & Created: {} (ID: {})", value, new_id);
        Ok(new_id)
    }

    pub fn reject_pending_action(&self, action_id: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("UPDATE pending_actions SET status = 'Rejected' WHERE id = ?", params![action_id])?;
        info!("Action Rejected: {}", action_id);
        Ok(())
    }

    pub fn upsert_island(
        &self, 
        table: &str, name: &str, path: &str,
        relations: &HashMap<String, Option<String>>,
        aggregations: &HashMap<String, f64>
    ) -> Result<()> {
        let conn = self.conn.lock().unwrap();

        let query_select = format!("SELECT id FROM {} WHERE name = ?", table);
        let project_id: String = {
            let mut stmt = conn.prepare(&query_select)?;
            let mut rows = stmt.query(params![name])?;
            if let Some(row) = rows.next()? { row.get(0)? } else { Uuid::new_v4().to_string() }
        };

        conn.execute(&format!("DELETE FROM {} WHERE id = ?", table), params![project_id])?;

        let mut final_cols = vec!["id".to_string(), "name".to_string(), "path".to_string(), "updated_at".to_string()];
        let now = chrono::Local::now().to_rfc3339();
        let mut final_vals = vec![format!("'{}'", project_id), format!("'{}'", name), format!("'{}'", path), format!("'{}'", now)];
        
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

        let query = format!("INSERT INTO {} ({}) VALUES ({})", table, final_cols.join(", "), final_vals.join(", "));
        conn.execute(&query, [])?;
        Ok(())
    }

    pub fn fetch_pending_actions(&self) -> Result<Vec<JsonValue>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT * FROM pending_actions WHERE status = 'Pending'")?;
        
        let col_names: Vec<String> = stmt.column_names().into_iter().map(|s| s.to_string()).collect();
        let rows = stmt.query_map([], |row| {
             let mut map = Map::new();
             for i in 0..col_names.len() {
                 // Fetch as owned SqlValue
                 let val: SqlValue = row.get(i)?;
                 let json_val = match val {
                    SqlValue::Text(s) => {
                        if col_names[i] == "suggestions" {
                             serde_json::from_str(&s).unwrap_or(JsonValue::String(s))
                        } else {
                             JsonValue::String(s)
                        }
                    },
                    SqlValue::Null => JsonValue::Null,
                    _ => JsonValue::String("Unknown".to_string())
                 };
                 map.insert(col_names[i].clone(), json_val);
             }
             Ok(JsonValue::Object(map))
        })?;

        let mut res = Vec::new();
        for r in rows { res.push(r?); }
        Ok(res)
    }

    pub fn fetch_all_dynamic(&self, table: &str) -> Result<Vec<JsonValue>> {
        let conn = self.conn.lock().unwrap();
        let query = format!("SELECT * FROM {}", table);
        let mut stmt = conn.prepare(&query)?;
        
        let col_names: Vec<String> = stmt.column_names().into_iter().map(|s| s.to_string()).collect();
        let rows = stmt.query_map([], |row| {
            let mut map = Map::new();
            for i in 0..col_names.len() {
                // IMPORTANT FIX: Use row.get() to get OWNED value, not get_ref()
                let val: SqlValue = row.get(i)?;
                
                let json_val = match val {
                    SqlValue::Null => JsonValue::Null,
                    SqlValue::Integer(i) => JsonValue::Number(i.into()),
                    SqlValue::Real(f) => if let Some(n) = serde_json::Number::from_f64(f) { JsonValue::Number(n) } else { JsonValue::Null },
                    SqlValue::Text(s) => JsonValue::String(s),
                    SqlValue::Blob(_) => JsonValue::String("<BINARY>".to_string()),
                };
                map.insert(col_names[i].clone(), json_val);
            }
            Ok(JsonValue::Object(map))
        })?;
        
        let mut results = Vec::new();
        for r in rows { results.push(r?); }
        Ok(results)
    }
    
    // Backwards compatibility if needed
    pub fn upsert_entity(&self, table: &str, key_field: &str, value: &str) -> Result<String> {
        let conn = self.conn.lock().unwrap();
        let query_select = format!("SELECT id FROM {} WHERE {} = ?", table, key_field);
        let mut stmt = conn.prepare(&query_select)?;
        if let Some(row) = stmt.query(params![value])?.next()? { return Ok(row.get(0)?); }
        let new_id = Uuid::new_v4().to_string();
        conn.execute(&format!("INSERT INTO {} (id, {}) VALUES (?, ?)", table, key_field), params![new_id, value])?;
        Ok(new_id)
    }
}

