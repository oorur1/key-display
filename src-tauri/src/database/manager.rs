use rusqlite::{Connection, OptionalExtension, Result};
use std::path::PathBuf;
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
      id INTEGER PRIMARY KEY AUTOINCREMENT,
      date TEXT NOT NULL UNIQUE,
      notes_count INTEGER NOT NULL
    )";

    self.conn.execute(sql, [])
    .map_err(|e| DatabaseError::InitializeError(format!("Failed to create database{}", e)))?;
    Ok(())
  }
  
  pub fn insert(&self, date: &str, notes_count: i32) -> Result<(), DatabaseError>{
    self.conn.execute(
      "INSERT INTO statistics (date, notes_count) VALUES (?1, ?2)",
      [date, &notes_count.to_string()],)
    .map_err(|e| DatabaseError::QueryError(format!("Failed to insert: {}", e)))?;
    Ok(())
  }
  
  pub fn get(&self, date: &str) -> Result<Option<i32>, DatabaseError>{
    let mut stmt = self.conn.prepare("SELECT notes_count FROM statistics WHERE date = ?1")
      .map_err(|e| DatabaseError::QueryError(format!("Failed to prepare: {}", e)))?;
    let result = stmt.query_row([date], |row| row.get(0)).optional()
      .map_err(|e| DatabaseError::QueryError(format!("Failed to query: {}", e)))?;
    
    Ok(result)
  }
  
  pub fn update(&self, date: &str, notes_count: i32) -> Result<(), DatabaseError>{
    self.conn.execute(
      "UPDATE statistics SET notes_count = ?1 WHERE date = ?2",
      [&notes_count.to_string(), date],
    )
    .map_err(|e| DatabaseError::QueryError(format!("Failed to update: {}", e)))?;
    Ok(())
  }

  pub fn delete(&self) -> Result<(), DatabaseError>{
    Ok(())
  }
}

#[cfg(test)]
mod tests{
  use super::*;
  use std::fs;
  // テスト用の一時データベースを作成
    fn setup_test_db(test_name: &str) -> (DatabaseManager, PathBuf) {
        let test_db_path = PathBuf::from(format!("test_{}.db", test_name));
        
        // 既存のテストDBがあれば削除
        if test_db_path.exists() {
            fs::remove_file(&test_db_path).unwrap();
        }
        
        let mut db = DatabaseManager::new(test_db_path.clone()).unwrap();
        db.initialize().unwrap();
        
        (db, test_db_path)
    }
    
    // テスト後のクリーンアップ
    fn cleanup_test_db(path: PathBuf) {
        if path.exists() {
            fs::remove_file(path).unwrap();
        }
    }
    
    #[test]
    fn test_database_creation() {
        let (db, path) = setup_test_db("creation");
        assert!(path.exists());
        cleanup_test_db(path);
    }
    
    #[test]
    fn test_insert_and_get() {
        let (db, path) = setup_test_db("insert_and_get");
        
        // データの挿入
        let result = db.insert("2025-01-01", 100);
        assert!(result.is_ok());
        
        // データの取得
        let count = db.get("2025-01-01").unwrap();
        assert_eq!(count, Some(100));
        
        cleanup_test_db(path);
    }
    
    #[test]
    fn test_update() {
        let (db, path) = setup_test_db("update");
        
        // 初期データ挿入
        db.insert("2025-01-01", 100).unwrap();
        
        // データの更新
        let result = db.update("2025-01-01", 200);
        assert!(result.is_ok());
        
        // 更新後の確認
        let count = db.get("2025-01-01").unwrap();
        assert_eq!(count, Some(200));
        
        cleanup_test_db(path);
    }
    
    #[test]
    fn test_get_nonexistent_data() {
        let (db, path) = setup_test_db("nonexistent");
        
        // 存在しないデータの取得
        let count = db.get("2099-12-31").unwrap();
        assert_eq!(count, None);
        
        cleanup_test_db(path);
    }
    
    #[test]
    fn test_insert_duplicate() {
        let (db, path) = setup_test_db("duplicate");
        
        // 同じ日付で2回挿入
        db.insert("2025-01-01", 100).unwrap();
        let result = db.insert("2025-01-01", 200);
        
        // UNIQUE制約違反でエラーになるはず
        assert!(result.is_err());
        
        cleanup_test_db(path);
    }
    
    #[test]
    fn test_multiple_dates() {
        let (db, path) = setup_test_db("multiple");
        
        // 複数の日付でデータを挿入
        db.insert("2025-01-01", 100).unwrap();
        db.insert("2025-01-02", 200).unwrap();
        db.insert("2025-01-03", 300).unwrap();
        
        // それぞれ確認
        assert_eq!(db.get("2025-01-01").unwrap(), Some(100));
        assert_eq!(db.get("2025-01-02").unwrap(), Some(200));
        assert_eq!(db.get("2025-01-03").unwrap(), Some(300));
        
        cleanup_test_db(path);
    }
}