use rusqlite::{Connection, params};
use uuid::Uuid;
use crate::config::{Config, Definition};
use anyhow::{Result, Context};
use tracing::{info, debug};
use std::sync::Mutex;

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
        for def in &config.definitions {
            if let Definition::Cloud(cloud_def) = def {
                let table_name = &cloud_def.name;
                // Basic Schema: id (UUID), and dynamic fields (TEXT)
                
                let mut columns = vec!["id TEXT PRIMARY KEY".to_string()];
                for field in &cloud_def.fields {
                    // Sanitize field names? Assuming safe from config for now.
                    columns.push(format!("{} TEXT", field));
                }
                
                let query = format!(
                    "CREATE TABLE IF NOT EXISTS {} ({})",
                    table_name,
                    columns.join(", ")
                );
                
                debug!("Executing: {}", query);
                conn.execute(&query, [])?;
                info!("Initialized Cloud Table: {}", table_name);
            }
        }
        Ok(())
    }

    // Implicit Creation Logic
    pub fn upsert_entity(&self, table: &str, key_field: &str, value: &str) -> Result<String> {
        let conn = self.conn.lock().unwrap();
        
        // 1. Check existence
        // SECURITY: table and key_field are internal/config derived, so injection risk is lower but still should be careful.
        // value is user input (from files).
        let query_select = format!("SELECT id FROM {} WHERE {} = ?", table, key_field);
        let mut stmt = conn.prepare(&query_select)?;
        let mut rows = stmt.query(params![value])?;

        if let Some(row) = rows.next()? {
            let id: String = row.get(0)?;
            debug!("Entity exists in {}: {} (id: {})", table, value, id);
            Ok(id)
        } else {
            // 2. Insert new
            let new_id = Uuid::new_v4().to_string();
            let query_insert = format!("INSERT INTO {} (id, {}) VALUES (?, ?)", table, key_field);
            conn.execute(&query_insert, params![new_id, value])?;
            info!("Created Implicit Entity in {}: {} (id: {})", table, value, new_id);
            Ok(new_id)
        }
    }

    pub fn get_all(&self, table: &str) -> Result<Vec<(String, String)>> {
        let conn = self.conn.lock().unwrap();
        let query = format!("SELECT id, name FROM {}", table);
        let mut stmt = conn.prepare(&query)?;
        
        let rows = stmt.query_map([], |row| {
            Ok((row.get(0)?, row.get(1)?))
        })?;

        let mut results = Vec::new();
        for row in rows {
            results.push(row?);
        }
        Ok(results)
    }
}