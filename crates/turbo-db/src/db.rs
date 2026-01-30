//! Database connection and query execution.

use crate::{DbError, QueryResult, Row, Value};
use serde::de::DeserializeOwned;

/// SQLite database connection.
///
/// Provides type-safe query execution with automatic result deserialization.
pub struct Db {
    #[cfg(target_arch = "wasm32")]
    conn: spin_sdk::sqlite::Connection,
    #[cfg(not(target_arch = "wasm32"))]
    _phantom: std::marker::PhantomData<()>,
}

impl Db {
    /// Open the default SQLite database.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let db = Db::open_default()?;
    /// ```
    #[cfg(target_arch = "wasm32")]
    pub fn open_default() -> Result<Self, DbError> {
        let conn = spin_sdk::sqlite::Connection::open_default()
            .map_err(|e| DbError::OpenError(e.to_string()))?;
        Ok(Self { conn })
    }

    /// Open a named SQLite database.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let db = Db::open("my-database")?;
    /// ```
    #[cfg(target_arch = "wasm32")]
    pub fn open(name: &str) -> Result<Self, DbError> {
        let conn = spin_sdk::sqlite::Connection::open(name)
            .map_err(|e| DbError::OpenError(e.to_string()))?;
        Ok(Self { conn })
    }

    /// Execute a SQL statement that doesn't return rows.
    ///
    /// Use this for INSERT, UPDATE, DELETE, CREATE TABLE, etc.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// db.execute(
    ///     "INSERT INTO products (name, price) VALUES (?, ?)",
    ///     params!["Rust Book", 49.99]
    /// )?;
    /// ```
    #[cfg(target_arch = "wasm32")]
    pub fn execute(&self, sql: &str, params: &[Value]) -> Result<(), DbError> {
        let spin_params: Vec<spin_sdk::sqlite::Value> = params
            .iter()
            .map(|v| match v {
                Value::Null => spin_sdk::sqlite::Value::Null,
                Value::Integer(i) => spin_sdk::sqlite::Value::Integer(*i),
                Value::Real(f) => spin_sdk::sqlite::Value::Real(*f),
                Value::Text(s) => spin_sdk::sqlite::Value::Text(s.clone()),
                Value::Blob(b) => spin_sdk::sqlite::Value::Blob(b.clone()),
            })
            .collect();

        self.conn
            .execute(sql, spin_params.as_slice())
            .map_err(|e| DbError::QueryError(e.to_string()))?;

        Ok(())
    }

    /// Execute a SQL query and return raw results.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let result = db.query("SELECT * FROM products WHERE price < ?", params![100.0])?;
    /// for row in result.iter() {
    ///     let name = row.get("name").and_then(|v| v.as_text());
    ///     println!("Product: {:?}", name);
    /// }
    /// ```
    #[cfg(target_arch = "wasm32")]
    pub fn query(&self, sql: &str, params: &[Value]) -> Result<QueryResult, DbError> {
        let spin_params: Vec<spin_sdk::sqlite::Value> = params
            .iter()
            .map(|v| match v {
                Value::Null => spin_sdk::sqlite::Value::Null,
                Value::Integer(i) => spin_sdk::sqlite::Value::Integer(*i),
                Value::Real(f) => spin_sdk::sqlite::Value::Real(*f),
                Value::Text(s) => spin_sdk::sqlite::Value::Text(s.clone()),
                Value::Blob(b) => spin_sdk::sqlite::Value::Blob(b.clone()),
            })
            .collect();

        let result = self
            .conn
            .execute(sql, spin_params.as_slice())
            .map_err(|e| DbError::QueryError(e.to_string()))?;

        let columns: Vec<String> = result.columns.iter().map(|c| c.to_string()).collect();

        let rows: Vec<Row> = result
            .rows
            .iter()
            .map(|row| {
                let values: Vec<Value> = row
                    .values
                    .iter()
                    .map(|v| match v {
                        spin_sdk::sqlite::Value::Null => Value::Null,
                        spin_sdk::sqlite::Value::Integer(i) => Value::Integer(*i),
                        spin_sdk::sqlite::Value::Real(f) => Value::Real(*f),
                        spin_sdk::sqlite::Value::Text(s) => Value::Text(s.clone()),
                        spin_sdk::sqlite::Value::Blob(b) => Value::Blob(b.clone()),
                    })
                    .collect();
                Row::new(columns.clone(), values)
            })
            .collect();

        Ok(QueryResult::new(columns, rows))
    }

    /// Execute a SQL query and deserialize results into a vector.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// #[derive(Deserialize)]
    /// struct Product {
    ///     id: i64,
    ///     name: String,
    ///     price: f64,
    /// }
    ///
    /// let products: Vec<Product> = db.query_as(
    ///     "SELECT id, name, price FROM products WHERE price < ?",
    ///     params![100.0]
    /// )?;
    /// ```
    #[cfg(target_arch = "wasm32")]
    pub fn query_as<T: DeserializeOwned>(
        &self,
        sql: &str,
        params: &[Value],
    ) -> Result<Vec<T>, DbError> {
        let result = self.query(sql, params)?;
        result.deserialize_all()
    }

    /// Execute a SQL query and return a single row.
    ///
    /// Returns an error if no rows are returned.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let product: Product = db.query_one(
    ///     "SELECT * FROM products WHERE id = ?",
    ///     params![1]
    /// )?;
    /// ```
    #[cfg(target_arch = "wasm32")]
    pub fn query_one<T: DeserializeOwned>(
        &self,
        sql: &str,
        params: &[Value],
    ) -> Result<T, DbError> {
        let result = self.query(sql, params)?;
        result.first().ok_or(DbError::NotFound)?.deserialize()
    }

    /// Execute a SQL query and return an optional single row.
    ///
    /// Returns `None` if no rows are returned.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let product: Option<Product> = db.query_optional(
    ///     "SELECT * FROM products WHERE id = ?",
    ///     params![1]
    /// )?;
    /// ```
    #[cfg(target_arch = "wasm32")]
    pub fn query_optional<T: DeserializeOwned>(
        &self,
        sql: &str,
        params: &[Value],
    ) -> Result<Option<T>, DbError> {
        let result = self.query(sql, params)?;
        match result.first() {
            Some(row) => Ok(Some(row.deserialize()?)),
            None => Ok(None),
        }
    }

    // Non-WASM stubs for development/testing
    #[cfg(not(target_arch = "wasm32"))]
    pub fn open_default() -> Result<Self, DbError> {
        Ok(Self {
            _phantom: std::marker::PhantomData,
        })
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn open(_name: &str) -> Result<Self, DbError> {
        Ok(Self {
            _phantom: std::marker::PhantomData,
        })
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn execute(&self, _sql: &str, _params: &[Value]) -> Result<(), DbError> {
        Ok(())
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn query(&self, _sql: &str, _params: &[Value]) -> Result<QueryResult, DbError> {
        Ok(QueryResult::new(vec![], vec![]))
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn query_as<T: DeserializeOwned>(
        &self,
        _sql: &str,
        _params: &[Value],
    ) -> Result<Vec<T>, DbError> {
        Ok(vec![])
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn query_one<T: DeserializeOwned>(
        &self,
        _sql: &str,
        _params: &[Value],
    ) -> Result<T, DbError> {
        Err(DbError::NotFound)
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn query_optional<T: DeserializeOwned>(
        &self,
        _sql: &str,
        _params: &[Value],
    ) -> Result<Option<T>, DbError> {
        Ok(None)
    }
}
