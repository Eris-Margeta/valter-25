use rusqlite::{Connection, params, types::Value as SqlValue};
use uuid::Uuid;
use crate::config::Config;
use anyhow::{Result, Context};
use tracing::{info, debug, warn};
use std::sync::Mutex;
use std::collections::HashMap;
use serde_json::{Value as JsonValue, Map};

pub struct SqliteManager {
    conn: Mutex<Connection>,
}

impl SqliteManager {
    pub fn new(path: &str) -> Result<Self> {
        let conn = Connection::open(path).context("Failed to open SQLite DB")?;
        Ok(Self { conn: Mutex::new(conn) })
    }

    pub fn init_schema(&self, config: &Config) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        
        // 1. Inicijalizacija CLOUDS (Klijenti, Operateri...)
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
            } else {
                info!("Cloud Table Ready: {}", table_name);
            }
        }

        // 2. Inicijalizacija ISLANDS (Projekti)
        for island_def in &config.islands {
            let table_name = &island_def.name;
            let mut columns = vec![
                "id TEXT PRIMARY KEY".to_string(),
                "name TEXT".to_string(),
                "path TEXT".to_string(),
                "status TEXT".to_string(),
                "updated_at TEXT".to_string()
            ];

            // Relacije (Foreign Keys)
            for rel in &island_def.relations {
                columns.push(format!("{} TEXT", rel.field));
            }

            // Agregacije (Virtualne kolone s brojkama)
            for agg in &island_def.aggregations {
                columns.push(format!("{} REAL", agg.name));
            }

            let query = format!("CREATE TABLE IF NOT EXISTS {} ({})", table_name, columns.join(", "));
            if let Err(e) = conn.execute(&query, []) {
                warn!("Error creating Island table {}: {}", table_name, e);
            } else {
                info!("Island Table Ready: {}", table_name);
            }
        }

        Ok(())
    }

    pub fn upsert_entity(&self, table: &str, key_field: &str, value: &str) -> Result<String> {
        let conn = self.conn.lock().unwrap();
        
        let query_select = format!("SELECT id FROM {} WHERE {} = ?", table, key_field);
        let mut stmt = conn.prepare(&query_select)?;
        let mut rows = stmt.query(params![value])?;

        if let Some(row) = rows.next()? {
            Ok(row.get(0)?)
        } else {
            let new_id = Uuid::new_v4().to_string();
            let query_insert = format!("INSERT INTO {} (id, {}) VALUES (?, ?)", table, key_field);
            conn.execute(&query_insert, params![new_id, value])?;
            info!("Created Entity in {}: {} (ID: {})", table, value, new_id);
            Ok(new_id)
        }
    }

    pub fn upsert_island(
        &self, 
        table: &str, 
        name: &str, 
        path: &str,
        relations: &HashMap<String, String>, 
        aggregations: &HashMap<String, f64>
    ) -> Result<()> {
        let conn = self.conn.lock().unwrap();

        // 1. Provjeri ID po imenu
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

        // 2. Brišemo stari zapis da bi ubacili novi sa svim svježim relacijama i agregacijama
        conn.execute(&format!("DELETE FROM {} WHERE id = ?", table), params![project_id])?;

        // 3. Pripremamo INSERT
        let mut final_cols = vec!["id".to_string(), "name".to_string(), "path".to_string(), "updated_at".to_string()];
        let now = chrono::Local::now().to_rfc3339();
        
        // Vrijednosti moraju biti escape-ane ako su stringovi. 
        // Ovdje koristimo jednostavni format! za ovaj level prototipa.
        let mut final_vals = vec![
            format!("'{}'", project_id), 
            format!("'{}'", name), 
            format!("'{}'", path), 
            format!("'{}'", now)
        ];
        
        for (k, v) in relations {
             final_cols.push(k.clone());
             final_vals.push(format!("'{}'", v));
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

        debug!("Upserting Island Query: {}", query);
        conn.execute(&query, [])?;
        
        info!("Island Synced: {}", name);
        Ok(())
    }

    /// Generički dohvat podataka za API (vraća JSON)
    pub fn fetch_all_dynamic(&self, table: &str) -> Result<Vec<JsonValue>> {
        let conn = self.conn.lock().unwrap();
        
        let query = format!("SELECT * FROM {}", table);
        let mut stmt = conn.prepare(&query)?;
        
        let col_names: Vec<String> = stmt.column_names().into_iter().map(|s| s.to_string()).collect();
        let col_count = col_names.len();

        let rows = stmt.query_map([], |row| {
            let mut map = Map::new();
            for i in 0..col_count {
                let col_name = &col_names[i];
                let val = row.get_ref(i)?;
                
                let json_val = match val {
                    SqlValue::Null => JsonValue::Null,
                    SqlValue::Integer(i) => JsonValue::Number((*i).into()),
                    SqlValue::Real(f) => {
                        if let Some(n) = serde_json::Number::from_f64(*f) {
                            JsonValue::Number(n)
                        } else {
                            JsonValue::Null
                        }
                    },
                    SqlValue::Text(s) => JsonValue::String(s.clone()),
                    SqlValue::Blob(_) => JsonValue::String("<BINARY>".to_string()),
                };
                
                map.insert(col_name.clone(), json_val);
            }
            Ok(JsonValue::Object(map))
        })?;

        let mut results = Vec::new();
        for r in rows {
            if let Ok(json) = r {
                results.push(json);
            }
        }
        
        Ok(results)
    }
}

