use rusqlite::{Connection, Result};
use std::path::{Path, PathBuf};
use super::error::DatabaseError;

#[derive(Debug)]
pub struct DatabaseManager{
  path: PathBuf,
  conn: Connection,
}

impl DatabaseManager{
  pub fn new(path_buf: PathBuf) -> Result<DatabaseManager, DatabaseError> {
    let conn = Connection::open(&path_buf).map_err(|e| DatabaseError::ConnectionError(format!("Failed to open database {}", e)))?;
    Ok(DatabaseManager {
      path: path_buf,
      conn,
    })
  }
  
  pub fn initialize(&mut self) -> Result<(), DatabaseError>{
    let sql = 
    "CREATE TABLE IF NOT EXISTS statistics(
      date TEXT PRIMARY KEY,
      notes_count INTEGER NOT NULL,
    )";

    self.conn.execute(sql, [])
    .map_err(|e| DatabaseError::InitializeError(format!("Failed to create database{}", e)))?;
    Ok(())
  }
}