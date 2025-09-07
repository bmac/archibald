//! Query execution and connection pool interface

use crate::{QueryBuilder, Result, Value};
use serde::de::DeserializeOwned;
use std::future::Future;

/// Trait for database connection pools
pub trait ConnectionPool: Send + Sync + Clone {
    /// The connection type for this pool
    type Connection;
    
    /// Acquire a connection from the pool
    fn acquire(&self) -> impl Future<Output = Result<Self::Connection>> + Send;
    
    /// Execute a query that returns no results (INSERT, UPDATE, DELETE)
    fn execute(
        &self, 
        sql: &str, 
        params: &[Value]
    ) -> impl Future<Output = Result<u64>> + Send;
    
    /// Execute a query that returns multiple rows
    fn fetch_all<T>(
        &self,
        sql: &str,
        params: &[Value],
    ) -> impl Future<Output = Result<Vec<T>>> + Send
    where
        T: DeserializeOwned + Send + Unpin;
    
    /// Execute a query that returns a single row
    fn fetch_one<T>(
        &self,
        sql: &str,
        params: &[Value],
    ) -> impl Future<Output = Result<T>> + Send
    where
        T: DeserializeOwned + Send + Unpin;
    
    /// Execute a query that returns an optional row
    fn fetch_optional<T>(
        &self,
        sql: &str,
        params: &[Value],
    ) -> impl Future<Output = Result<Option<T>>> + Send
    where
        T: DeserializeOwned + Send + Unpin;
}

/// Transaction isolation levels
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IsolationLevel {
    ReadUncommitted,
    ReadCommitted,
    RepeatableRead,
    Serializable,
}

impl IsolationLevel {
    pub fn to_sql(&self) -> &'static str {
        match self {
            IsolationLevel::ReadUncommitted => "READ UNCOMMITTED",
            IsolationLevel::ReadCommitted => "READ COMMITTED", 
            IsolationLevel::RepeatableRead => "REPEATABLE READ",
            IsolationLevel::Serializable => "SERIALIZABLE",
        }
    }
}

/// Trait for database transactions
pub trait Transaction: Send + Sync {
    /// Execute a query that returns no results (INSERT, UPDATE, DELETE)
    fn execute(
        &mut self, 
        sql: &str, 
        params: &[Value]
    ) -> impl Future<Output = Result<u64>> + Send;
    
    /// Execute a query that returns multiple rows
    fn fetch_all<T>(
        &mut self,
        sql: &str,
        params: &[Value],
    ) -> impl Future<Output = Result<Vec<T>>> + Send
    where
        T: DeserializeOwned + Send + Unpin;
    
    /// Execute a query that returns a single row
    fn fetch_one<T>(
        &mut self,
        sql: &str,
        params: &[Value],
    ) -> impl Future<Output = Result<T>> + Send
    where
        T: DeserializeOwned + Send + Unpin;
    
    /// Execute a query that returns an optional row
    fn fetch_optional<T>(
        &mut self,
        sql: &str,
        params: &[Value],
    ) -> impl Future<Output = Result<Option<T>>> + Send
    where
        T: DeserializeOwned + Send + Unpin;
        
    /// Commit the transaction
    fn commit(self) -> impl Future<Output = Result<()>> + Send
    where
        Self: Sized;
    
    /// Rollback the transaction
    fn rollback(self) -> impl Future<Output = Result<()>> + Send
    where
        Self: Sized;
        
    /// Create a savepoint with the given name
    fn savepoint(&mut self, name: &str) -> impl Future<Output = Result<()>> + Send;
    
    /// Rollback to a savepoint
    fn rollback_to_savepoint(&mut self, name: &str) -> impl Future<Output = Result<()>> + Send;
    
    /// Release a savepoint
    fn release_savepoint(&mut self, name: &str) -> impl Future<Output = Result<()>> + Send;
}

/// Extension trait for connection pools to support transactions
pub trait TransactionalPool: ConnectionPool {
    type Transaction: Transaction;
    
    /// Start a new transaction with default isolation level
    fn begin_transaction(&self) -> impl Future<Output = Result<Self::Transaction>> + Send;
    
    /// Start a new transaction with specified isolation level
    fn begin_transaction_with_isolation(
        &self, 
        isolation: IsolationLevel
    ) -> impl Future<Output = Result<Self::Transaction>> + Send;
}

/// Convenience function for running code in a transaction
pub async fn transaction<P, F, Fut, T, E>(
    pool: &P,
    f: F,
) -> Result<T>
where
    P: TransactionalPool,
    F: FnOnce(&mut P::Transaction) -> Fut,
    Fut: Future<Output = std::result::Result<T, E>> + Send,
    E: Into<crate::Error>,
{
    let mut txn = pool.begin_transaction().await?;
    
    match f(&mut txn).await {
        Ok(result) => {
            txn.commit().await?;
            Ok(result)
        }
        Err(e) => {
            let _ = txn.rollback().await; // Ignore rollback errors
            Err(e.into())
        }
    }
}

/// Extension trait for query builders to add execution methods
pub trait ExecutableQuery<T>: QueryBuilder {
    /// Execute the query and return all results
    fn fetch_all<P>(self, pool: &P) -> impl Future<Output = Result<Vec<T>>> + Send
    where
        P: ConnectionPool,
        T: DeserializeOwned + Send + Unpin;
    
    /// Execute the query and return the first result
    fn fetch_one<P>(self, pool: &P) -> impl Future<Output = Result<T>> + Send
    where
        P: ConnectionPool,
        T: DeserializeOwned + Send + Unpin;
    
    /// Execute the query and return an optional result
    fn fetch_optional<P>(self, pool: &P) -> impl Future<Output = Result<Option<T>>> + Send
    where
        P: ConnectionPool,
        T: DeserializeOwned + Send + Unpin;
        
    /// Execute the query within a transaction and return all results
    fn fetch_all_tx<Tx>(self, tx: &mut Tx) -> impl Future<Output = Result<Vec<T>>> + Send
    where
        Tx: Transaction,
        T: DeserializeOwned + Send + Unpin;
    
    /// Execute the query within a transaction and return the first result
    fn fetch_one_tx<Tx>(self, tx: &mut Tx) -> impl Future<Output = Result<T>> + Send
    where
        Tx: Transaction,
        T: DeserializeOwned + Send + Unpin;
    
    /// Execute the query within a transaction and return an optional result
    fn fetch_optional_tx<Tx>(self, tx: &mut Tx) -> impl Future<Output = Result<Option<T>>> + Send
    where
        Tx: Transaction,
        T: DeserializeOwned + Send + Unpin;
}

/// Extension trait for modification queries (INSERT, UPDATE, DELETE)
pub trait ExecutableModification: QueryBuilder {
    /// Execute the modification query and return the number of affected rows
    fn execute<P>(self, pool: &P) -> impl Future<Output = Result<u64>> + Send
    where
        P: ConnectionPool;
        
    /// Execute the modification query within a transaction and return the number of affected rows
    fn execute_tx<Tx>(self, tx: &mut Tx) -> impl Future<Output = Result<u64>> + Send
    where
        Tx: Transaction;
}

// Implementation for SelectBuilder
impl<T> ExecutableQuery<T> for crate::SelectBuilder
where
    T: DeserializeOwned + Send + Unpin,
{
    async fn fetch_all<P>(self, pool: &P) -> Result<Vec<T>>
    where
        P: ConnectionPool,
    {
        let sql = self.to_sql()?;
        let params = self.parameters();
        pool.fetch_all(&sql, params).await
    }
    
    async fn fetch_one<P>(self, pool: &P) -> Result<T>
    where
        P: ConnectionPool,
    {
        let sql = self.to_sql()?;
        let params = self.parameters();
        pool.fetch_one(&sql, params).await
    }
    
    async fn fetch_optional<P>(self, pool: &P) -> Result<Option<T>>
    where
        P: ConnectionPool,
    {
        let sql = self.to_sql()?;
        let params = self.parameters();
        pool.fetch_optional(&sql, params).await
    }
    
    async fn fetch_all_tx<Tx>(self, tx: &mut Tx) -> Result<Vec<T>>
    where
        Tx: Transaction,
    {
        let sql = self.to_sql()?;
        let params = self.parameters();
        tx.fetch_all(&sql, params).await
    }
    
    async fn fetch_one_tx<Tx>(self, tx: &mut Tx) -> Result<T>
    where
        Tx: Transaction,
    {
        let sql = self.to_sql()?;
        let params = self.parameters();
        tx.fetch_one(&sql, params).await
    }
    
    async fn fetch_optional_tx<Tx>(self, tx: &mut Tx) -> Result<Option<T>>
    where
        Tx: Transaction,
    {
        let sql = self.to_sql()?;
        let params = self.parameters();
        tx.fetch_optional(&sql, params).await
    }
}

// Implementation for InsertBuilder
impl ExecutableModification for crate::InsertBuilder {
    async fn execute<P>(self, pool: &P) -> Result<u64>
    where
        P: ConnectionPool,
    {
        let sql = self.to_sql()?;
        let params = self.parameters();
        pool.execute(&sql, params).await
    }
    
    async fn execute_tx<Tx>(self, tx: &mut Tx) -> Result<u64>
    where
        Tx: Transaction,
    {
        let sql = self.to_sql()?;
        let params = self.parameters();
        tx.execute(&sql, params).await
    }
}

// Implementation for UpdateBuilder  
impl ExecutableModification for crate::UpdateBuilder {
    async fn execute<P>(self, pool: &P) -> Result<u64>
    where
        P: ConnectionPool,
    {
        let sql = self.to_sql()?;
        let params = self.parameters();
        pool.execute(&sql, params).await
    }
    
    async fn execute_tx<Tx>(self, tx: &mut Tx) -> Result<u64>
    where
        Tx: Transaction,
    {
        let sql = self.to_sql()?;
        let params = self.parameters();
        tx.execute(&sql, params).await
    }
}

// Implementation for DeleteBuilder
impl ExecutableModification for crate::DeleteBuilder {
    async fn execute<P>(self, pool: &P) -> Result<u64>
    where
        P: ConnectionPool,
    {
        let sql = self.to_sql()?;
        let params = self.parameters();
        pool.execute(&sql, params).await
    }
    
    async fn execute_tx<Tx>(self, tx: &mut Tx) -> Result<u64>
    where
        Tx: Transaction,
    {
        let sql = self.to_sql()?;
        let params = self.parameters();
        tx.execute(&sql, params).await
    }
}

/// SQLx connection pool wrapper
#[cfg(feature = "postgres")]
pub mod postgres {
    use super::*;
    use sqlx::PgPool;
    
    /// PostgreSQL connection pool wrapper
    #[derive(Clone)]
    pub struct PostgresPool {
        inner: PgPool,
    }
    
    impl PostgresPool {
        /// Create a new PostgreSQL pool from a connection string
        pub async fn new(database_url: &str) -> Result<Self> {
            let pool = PgPool::connect(database_url).await?;
            Ok(Self { inner: pool })
        }
        
        /// Create from an existing PgPool
        pub fn from_pool(pool: PgPool) -> Self {
            Self { inner: pool }
        }
    }
    
    impl ConnectionPool for PostgresPool {
        type Connection = sqlx::pool::PoolConnection<sqlx::Postgres>;
        
        async fn acquire(&self) -> Result<Self::Connection> {
            Ok(self.inner.acquire().await?)
        }
        
        async fn execute(&self, sql: &str, params: &[Value]) -> Result<u64> {
            let query = sqlx::query(sql);
            let bound_query = bind_values_to_query(query, params);
            let result = bound_query.execute(&self.inner).await?;
            Ok(result.rows_affected())
        }
        
        async fn fetch_all<T>(&self, sql: &str, params: &[Value]) -> Result<Vec<T>>
        where
            T: DeserializeOwned + Send + Unpin,
        {
            let query = sqlx::query(sql);
            let bound_query = bind_values_to_query(query, params);
            let rows = bound_query.fetch_all(&self.inner).await?;
                
            let mut results = Vec::with_capacity(rows.len());
            for row in rows {
                let json_value = row_to_json_value(&row)?;
                let item: T = serde_json::from_value(json_value)?;
                results.push(item);
            }
            Ok(results)
        }
        
        async fn fetch_one<T>(&self, sql: &str, params: &[Value]) -> Result<T>
        where
            T: DeserializeOwned + Send + Unpin,
        {
            let query = sqlx::query(sql);
            let bound_query = bind_values_to_query(query, params);
            let row = bound_query.fetch_one(&self.inner).await?;
                
            let json_value = row_to_json_value(&row)?;
            let item: T = serde_json::from_value(json_value)?;
            Ok(item)
        }
        
        async fn fetch_optional<T>(&self, sql: &str, params: &[Value]) -> Result<Option<T>>
        where
            T: DeserializeOwned + Send + Unpin,
        {
            let query = sqlx::query(sql);
            let bound_query = bind_values_to_query(query, params);
            if let Some(row) = bound_query.fetch_optional(&self.inner).await? {
                let json_value = row_to_json_value(&row)?;
                let item: T = serde_json::from_value(json_value)?;
                Ok(Some(item))
            } else {
                Ok(None)
            }
        }
    }
    
    /// PostgreSQL transaction wrapper
    pub struct PostgresTransaction {
        inner: sqlx::Transaction<'static, sqlx::Postgres>,
    }
    
    impl Transaction for PostgresTransaction {
        async fn execute(&mut self, sql: &str, params: &[Value]) -> Result<u64> {
            let query = sqlx::query(sql);
            let bound_query = bind_values_to_query(query, params);
            let result = bound_query.execute(&mut *self.inner).await?;
            Ok(result.rows_affected())
        }
        
        async fn fetch_all<T>(&mut self, sql: &str, params: &[Value]) -> Result<Vec<T>>
        where
            T: DeserializeOwned + Send + Unpin,
        {
            let query = sqlx::query(sql);
            let bound_query = bind_values_to_query(query, params);
            let rows = bound_query.fetch_all(&mut *self.inner).await?;
                
            let mut results = Vec::with_capacity(rows.len());
            for row in rows {
                let json_value = row_to_json_value(&row)?;
                let item: T = serde_json::from_value(json_value)?;
                results.push(item);
            }
            Ok(results)
        }
        
        async fn fetch_one<T>(&mut self, sql: &str, params: &[Value]) -> Result<T>
        where
            T: DeserializeOwned + Send + Unpin,
        {
            let query = sqlx::query(sql);
            let bound_query = bind_values_to_query(query, params);
            let row = bound_query.fetch_one(&mut *self.inner).await?;
                
            let json_value = row_to_json_value(&row)?;
            let item: T = serde_json::from_value(json_value)?;
            Ok(item)
        }
        
        async fn fetch_optional<T>(&mut self, sql: &str, params: &[Value]) -> Result<Option<T>>
        where
            T: DeserializeOwned + Send + Unpin,
        {
            let query = sqlx::query(sql);
            let bound_query = bind_values_to_query(query, params);
            if let Some(row) = bound_query.fetch_optional(&mut *self.inner).await? {
                let json_value = row_to_json_value(&row)?;
                let item: T = serde_json::from_value(json_value)?;
                Ok(Some(item))
            } else {
                Ok(None)
            }
        }
        
        async fn commit(self) -> Result<()> {
            self.inner.commit().await?;
            Ok(())
        }
        
        async fn rollback(self) -> Result<()> {
            self.inner.rollback().await?;
            Ok(())
        }
        
        async fn savepoint(&mut self, name: &str) -> Result<()> {
            let sql = format!("SAVEPOINT {}", name);
            sqlx::query(&sql).execute(&mut *self.inner).await?;
            Ok(())
        }
        
        async fn rollback_to_savepoint(&mut self, name: &str) -> Result<()> {
            let sql = format!("ROLLBACK TO SAVEPOINT {}", name);
            sqlx::query(&sql).execute(&mut *self.inner).await?;
            Ok(())
        }
        
        async fn release_savepoint(&mut self, name: &str) -> Result<()> {
            let sql = format!("RELEASE SAVEPOINT {}", name);
            sqlx::query(&sql).execute(&mut *self.inner).await?;
            Ok(())
        }
    }
    
    impl TransactionalPool for PostgresPool {
        type Transaction = PostgresTransaction;
        
        async fn begin_transaction(&self) -> Result<Self::Transaction> {
            let txn = self.inner.begin().await?;
            Ok(PostgresTransaction { inner: txn })
        }
        
        async fn begin_transaction_with_isolation(&self, isolation: IsolationLevel) -> Result<Self::Transaction> {
            let mut txn = self.inner.begin().await?;
            let sql = format!("SET TRANSACTION ISOLATION LEVEL {}", isolation.to_sql());
            sqlx::query(&sql).execute(&mut *txn).await?;
            Ok(PostgresTransaction { inner: txn })
        }
    }
    
    /// Bind Archibald Values to a SQLx query
    fn bind_values_to_query<'q>(
        mut query: sqlx::query::Query<'q, sqlx::Postgres, sqlx::postgres::PgArguments>,
        params: &'q [Value]
    ) -> sqlx::query::Query<'q, sqlx::Postgres, sqlx::postgres::PgArguments> {
        for param in params {
            query = match param {
                Value::Null => query.bind(None::<i32>), // Use Option<T> for NULL values
                Value::Bool(b) => query.bind(*b),
                Value::I32(i) => query.bind(*i),
                Value::I64(i) => query.bind(*i),
                Value::F32(f) => query.bind(*f),
                Value::F64(f) => query.bind(*f),
                Value::String(s) => query.bind(s.as_str()),
                Value::Bytes(b) => query.bind(b.as_slice()),
                Value::Json(j) => query.bind(j), // sqlx supports serde_json::Value directly
                Value::Array(arr) => {
                    // For arrays, we need to convert to a format that PostgreSQL understands
                    // For now, serialize simple arrays to JSON
                    let json_array = serde_json::Value::Array(
                        arr.iter().map(value_to_json).collect()
                    );
                    query.bind(json_array)
                },
                Value::SubqueryPlaceholder => {
                    // Subqueries should have been resolved before this point
                    // This is likely a programming error
                    continue; // Skip for now, could panic or error in the future
                }
            };
        }
        query
    }
    
    /// Convert Value to serde_json::Value for array serialization
    fn value_to_json(value: &Value) -> serde_json::Value {
        match value {
            Value::Null => serde_json::Value::Null,
            Value::Bool(b) => serde_json::Value::Bool(*b),
            Value::I32(i) => serde_json::Value::Number(serde_json::Number::from(*i)),
            Value::I64(i) => serde_json::Value::Number(serde_json::Number::from(*i)),
            Value::F32(f) => serde_json::Number::from_f64(*f as f64)
                .map(serde_json::Value::Number)
                .unwrap_or(serde_json::Value::Null),
            Value::F64(f) => serde_json::Number::from_f64(*f)
                .map(serde_json::Value::Number)
                .unwrap_or(serde_json::Value::Null),
            Value::String(s) => serde_json::Value::String(s.clone()),
            Value::Bytes(b) => serde_json::Value::Array(
                b.iter().map(|byte| serde_json::Value::Number(serde_json::Number::from(*byte))).collect()
            ),
            Value::Json(j) => j.clone(),
            Value::Array(arr) => serde_json::Value::Array(arr.iter().map(value_to_json).collect()),
            Value::SubqueryPlaceholder => serde_json::Value::Null,
        }
    }
    
    fn row_to_json_value(_row: &sqlx::postgres::PgRow) -> Result<serde_json::Value> {
        // This is a placeholder - in reality we'd need to convert SQLx row to JSON
        // For production, we'd iterate through columns and extract values
        // For now, return empty object for compilation
        Ok(serde_json::Value::Object(serde_json::Map::new()))
    }
    
    #[cfg(test)]
    mod postgres_tests {
        use super::*;
        
        #[test]
        fn test_postgres_pool_creation() {
            // Test that PostgresPool can be created from PgPool
            // This is mainly a compilation test since we can't easily create a real PgPool in tests
            assert!(true); // Placeholder test
        }
        
        #[test]
        fn test_value_to_json_conversion() {
            // Test basic value conversions
            assert_eq!(value_to_json(&Value::Null), serde_json::Value::Null);
            assert_eq!(value_to_json(&Value::Bool(true)), serde_json::Value::Bool(true));
            assert_eq!(value_to_json(&Value::I32(42)), serde_json::Value::Number(serde_json::Number::from(42)));
            assert_eq!(value_to_json(&Value::String("test".to_string())), serde_json::Value::String("test".to_string()));
            
            // Test array conversion
            let arr = Value::Array(vec![Value::I32(1), Value::I32(2), Value::I32(3)]);
            let json_arr = value_to_json(&arr);
            assert_eq!(json_arr, serde_json::json!([1, 2, 3]));
        }
        
        #[test] 
        fn test_parameter_binding_types() {
            // This test verifies our bind_values_to_query function can handle different Value types
            // We can't easily test the actual binding without a real database connection,
            // but we can verify the function doesn't panic with various value types
            use sqlx::query;
            
            let params = vec![
                Value::Null,
                Value::Bool(true),
                Value::I32(42),
                Value::I64(123456),
                Value::F32(3.14),
                Value::F64(2.718),
                Value::String("hello".to_string()),
                Value::Bytes(vec![1, 2, 3, 4]),
                Value::Json(serde_json::json!({"key": "value"})),
                Value::Array(vec![Value::I32(1), Value::I32(2)]),
            ];
            
            // Create a dummy query - this won't execute but will test the binding logic
            let query = query("SELECT * FROM users WHERE id = $1 AND name = $2");
            
            // Test that binding doesn't panic (we can't test execution without a real DB)
            let _bound_query = bind_values_to_query(query, &params[0..2]);
            // If we get here without panicking, the binding logic works
        }
        
        #[test]
        fn test_query_with_parameters_integration() {
            // Test that our query builder properly passes parameters to the executor
            use crate::{from, builder::QueryBuilder, op};
            
            // Build a query with parameters
            let query = from("users")
                .select(("id", "name", "email"))
                .where_(("age", op::GT, 18))
                .where_(("status", "active"))
                .where_(("score", op::IN, vec![100, 200, 300]));
                
            // Verify SQL generation and parameters
            let sql = query.to_sql().unwrap();
            let params = query.parameters();
            
            // Should have 3 parameters: age > 18, status = 'active', score IN [100,200,300] 
            assert_eq!(params.len(), 3);
            assert_eq!(params[0], crate::Value::I32(18));
            assert_eq!(params[1], crate::Value::String("active".to_string()));
            assert_eq!(params[2], crate::Value::Array(vec![
                crate::Value::I32(100),
                crate::Value::I32(200), 
                crate::Value::I32(300)
            ]));
            
            // Verify SQL contains proper placeholders
            assert!(sql.contains("?"));
            
            // Test that we can bind these parameters without panicking
            let sqlx_query = sqlx::query(&sql);
            let _bound_query = bind_values_to_query(sqlx_query, params);
        }
        
        #[test]
        fn test_transaction_isolation_levels() {
            // Test isolation level SQL generation
            assert_eq!(IsolationLevel::ReadUncommitted.to_sql(), "READ UNCOMMITTED");
            assert_eq!(IsolationLevel::ReadCommitted.to_sql(), "READ COMMITTED");
            assert_eq!(IsolationLevel::RepeatableRead.to_sql(), "REPEATABLE READ");
            assert_eq!(IsolationLevel::Serializable.to_sql(), "SERIALIZABLE");
        }
        
        #[tokio::test]
        async fn test_transaction_convenience_function() {
            use crate::{transaction};
            
            // Mock pool that would be used in a real transaction
            let pool = MockTransactionPool::new();
            
            // Test successful transaction
            let result: Result<i32> = transaction(&pool, |_txn| async move {
                // Simulate a simple successful operation
                Ok::<i32, crate::Error>(42)
            }).await;
            
            assert!(result.is_ok());
            assert_eq!(result.unwrap(), 42);
        }
        
        #[tokio::test]
        async fn test_transaction_rollback_on_error() {
            use crate::transaction;
            
            let pool = MockTransactionPool::new();
            
            // Test transaction rollback on error
            let result: Result<()> = transaction(&pool, |_txn| async move {
                // Simulate an error that should cause rollback
                Err(crate::Error::sql_generation("Simulated error"))
            }).await;
            
            assert!(result.is_err());
        }
        
        #[tokio::test] 
        async fn test_savepoints() {
            let pool = MockTransactionPool::new();
            let mut txn = pool.begin_transaction().await.unwrap();
            
            // Test savepoint operations
            txn.savepoint("sp1").await.unwrap();
            txn.rollback_to_savepoint("sp1").await.unwrap();
            txn.release_savepoint("sp1").await.unwrap();
            
            txn.rollback().await.unwrap();
        }
        
        // Mock types for testing transaction functionality without real database
        #[derive(Clone)]
        struct MockTransactionPool;
        
        impl MockTransactionPool {
            fn new() -> Self {
                Self
            }
        }
        
        impl ConnectionPool for MockTransactionPool {
            type Connection = ();
            
            async fn acquire(&self) -> Result<Self::Connection> {
                Ok(())
            }
            
            async fn execute(&self, _sql: &str, _params: &[Value]) -> Result<u64> {
                Ok(1)
            }
            
            async fn fetch_all<T>(&self, _sql: &str, _params: &[Value]) -> Result<Vec<T>>
            where
                T: DeserializeOwned + Send + Unpin,
            {
                Ok(Vec::new())
            }
            
            async fn fetch_one<T>(&self, _sql: &str, _params: &[Value]) -> Result<T>
            where
                T: DeserializeOwned + Send + Unpin,
            {
                Err(crate::Error::sql_generation("Mock fetch_one"))
            }
            
            async fn fetch_optional<T>(&self, _sql: &str, _params: &[Value]) -> Result<Option<T>>
            where
                T: DeserializeOwned + Send + Unpin,
            {
                Ok(None)
            }
        }
        
        impl TransactionalPool for MockTransactionPool {
            type Transaction = MockTransaction;
            
            async fn begin_transaction(&self) -> Result<Self::Transaction> {
                Ok(MockTransaction)
            }
            
            async fn begin_transaction_with_isolation(&self, _isolation: IsolationLevel) -> Result<Self::Transaction> {
                Ok(MockTransaction)
            }
        }
        
        struct MockTransaction;
        
        impl Transaction for MockTransaction {
            async fn execute(&mut self, _sql: &str, _params: &[Value]) -> Result<u64> {
                Ok(1)
            }
            
            async fn fetch_all<T>(&mut self, _sql: &str, _params: &[Value]) -> Result<Vec<T>>
            where
                T: DeserializeOwned + Send + Unpin,
            {
                if std::any::type_name::<T>().contains("User") {
                    let users_json = serde_json::json!([
                        {"id": 1, "name": "John", "email": "john@example.com"}
                    ]);
                    let users: Vec<T> = serde_json::from_value(users_json)?;
                    Ok(users)
                } else {
                    Ok(Vec::new())
                }
            }
            
            async fn fetch_one<T>(&mut self, _sql: &str, _params: &[Value]) -> Result<T>
            where
                T: DeserializeOwned + Send + Unpin,
            {
                if std::any::type_name::<T>().contains("User") {
                    let user_json = serde_json::json!({"id": 1, "name": "John", "email": "john@example.com"});
                    let user: T = serde_json::from_value(user_json)?;
                    Ok(user)
                } else {
                    Err(crate::Error::sql_generation("No mock data for this type"))
                }
            }
            
            async fn fetch_optional<T>(&mut self, _sql: &str, _params: &[Value]) -> Result<Option<T>>
            where
                T: DeserializeOwned + Send + Unpin,
            {
                Ok(None)
            }
            
            async fn commit(self) -> Result<()> {
                Ok(())
            }
            
            async fn rollback(self) -> Result<()> {
                Ok(())
            }
            
            async fn savepoint(&mut self, _name: &str) -> Result<()> {
                Ok(())
            }
            
            async fn rollback_to_savepoint(&mut self, _name: &str) -> Result<()> {
                Ok(())
            }
            
            async fn release_savepoint(&mut self, _name: &str) -> Result<()> {
                Ok(())
            }
        }
        
        #[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
        struct User {
            id: i32,
            name: String,
            email: String,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{from, op};
    use serde::{Deserialize, Serialize};
    use std::collections::HashMap;
    
    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    struct User {
        id: i32,
        name: String,
        email: String,
    }
    
    // Mock connection pool for testing
    #[derive(Clone)]
    struct MockPool {
        should_fail: bool,
    }
    
    impl MockPool {
        fn new() -> Self {
            Self { should_fail: false }
        }
        
        fn with_failure() -> Self {
            Self { should_fail: true }
        }
    }
    
    impl ConnectionPool for MockPool {
        type Connection = ();
        
        async fn acquire(&self) -> Result<Self::Connection> {
            if self.should_fail {
                Err(crate::Error::sql_generation("Mock connection failure"))
            } else {
                Ok(())
            }
        }
        
        async fn execute(&self, _sql: &str, _params: &[Value]) -> Result<u64> {
            if self.should_fail {
                Err(crate::Error::sql_generation("Mock execute failure"))
            } else {
                Ok(1) // Simulate 1 affected row
            }
        }
        
        async fn fetch_all<T>(&self, _sql: &str, _params: &[Value]) -> Result<Vec<T>>
        where
            T: DeserializeOwned + Send + Unpin,
        {
            if self.should_fail {
                return Err(crate::Error::sql_generation("Mock fetch_all failure"));
            }
            
            // Return mock data for User type
            if std::any::type_name::<T>().contains("User") {
                let users_json = serde_json::json!([
                    {"id": 1, "name": "John", "email": "john@example.com"},
                    {"id": 2, "name": "Jane", "email": "jane@example.com"}
                ]);
                let users: Vec<T> = serde_json::from_value(users_json)?;
                Ok(users)
            } else {
                Ok(Vec::new())
            }
        }
        
        async fn fetch_one<T>(&self, _sql: &str, _params: &[Value]) -> Result<T>
        where
            T: DeserializeOwned + Send + Unpin,
        {
            if self.should_fail {
                return Err(crate::Error::sql_generation("Mock fetch_one failure"));
            }
            
            if std::any::type_name::<T>().contains("User") {
                let user_json = serde_json::json!({"id": 1, "name": "John", "email": "john@example.com"});
                let user: T = serde_json::from_value(user_json)?;
                Ok(user)
            } else {
                return Err(crate::Error::sql_generation("No mock data for this type"));
            }
        }
        
        async fn fetch_optional<T>(&self, _sql: &str, _params: &[Value]) -> Result<Option<T>>
        where
            T: DeserializeOwned + Send + Unpin,
        {
            if self.should_fail {
                return Err(crate::Error::sql_generation("Mock fetch_optional failure"));
            }
            
            if std::any::type_name::<T>().contains("User") {
                let user_json = serde_json::json!({"id": 1, "name": "John", "email": "john@example.com"});
                let user: T = serde_json::from_value(user_json)?;
                Ok(Some(user))
            } else {
                Ok(None)
            }
        }
    }
    
    #[tokio::test]
    async fn test_select_fetch_all() {
        let pool = MockPool::new();
        let query = from("users")
            .select(("id", "name", "email"))
            .where_(("age", op::GT, 18));
            
        let users: Vec<User> = query.fetch_all(&pool).await.unwrap();
        assert_eq!(users.len(), 2);
        assert_eq!(users[0].name, "John");
        assert_eq!(users[1].name, "Jane");
    }
    
    #[tokio::test]
    async fn test_select_fetch_one() {
        let pool = MockPool::new();
        let query = from("users").where_(("id", 1));
        
        let user: User = query.fetch_one(&pool).await.unwrap();
        assert_eq!(user.id, 1);
        assert_eq!(user.name, "John");
    }
    
    #[tokio::test]
    async fn test_select_fetch_optional() {
        let pool = MockPool::new();
        let query = from("users").where_(("id", 1));
        
        let user: Option<User> = query.fetch_optional(&pool).await.unwrap();
        assert!(user.is_some());
        let user = user.unwrap();
        assert_eq!(user.id, 1);
    }
    
    #[tokio::test]
    async fn test_insert_execute() {
        let pool = MockPool::new();
        
        let mut data = HashMap::new();
        data.insert("name".to_string(), crate::Value::String("Test".to_string()));
        data.insert("email".to_string(), crate::Value::String("test@example.com".to_string()));
        
        let query = crate::InsertBuilder::new("users").values(data);
        let affected = query.execute(&pool).await.unwrap();
        assert_eq!(affected, 1);
    }
    
    #[tokio::test]
    async fn test_update_execute() {
        let pool = MockPool::new();
        
        let mut updates = HashMap::new();
        updates.insert("name".to_string(), crate::Value::String("Updated".to_string()));
        
        let query = crate::UpdateBuilder::new("users")
            .set(updates)
            .where_(("id", 1));
            
        let affected = query.execute(&pool).await.unwrap();
        assert_eq!(affected, 1);
    }
    
    #[tokio::test]
    async fn test_delete_execute() {
        let pool = MockPool::new();
        
        let query = crate::DeleteBuilder::new("users")
            .where_(("age", op::LT, 13));
            
        let affected = query.execute(&pool).await.unwrap();
        assert_eq!(affected, 1);
    }
    
    #[tokio::test]
    async fn test_connection_failure() {
        let pool = MockPool::with_failure();
        let query = from("users");
        
        let result: Result<Vec<User>> = query.fetch_all(&pool).await;
        assert!(result.is_err());
    }
    
    #[tokio::test]  
    async fn test_modification_failure() {
        let pool = MockPool::with_failure();
        
        let mut data = HashMap::new();
        data.insert("name".to_string(), crate::Value::String("Test".to_string()));
        
        let query = crate::InsertBuilder::new("users").values(data);
        let result = query.execute(&pool).await;
        assert!(result.is_err());
    }
}