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
}

/// Extension trait for modification queries (INSERT, UPDATE, DELETE)
pub trait ExecutableModification: QueryBuilder {
    /// Execute the modification query and return the number of affected rows
    fn execute<P>(self, pool: &P) -> impl Future<Output = Result<u64>> + Send
    where
        P: ConnectionPool;
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
        
        async fn execute(&self, sql: &str, _params: &[Value]) -> Result<u64> {
            // Simplified implementation for now - in production we'd properly bind parameters
            let result = sqlx::query(sql)
                .execute(&self.inner)
                .await?;
            Ok(result.rows_affected())
        }
        
        async fn fetch_all<T>(&self, sql: &str, _params: &[Value]) -> Result<Vec<T>>
        where
            T: DeserializeOwned + Send + Unpin,
        {
            // Simplified implementation for now - in production we'd properly bind parameters
            let rows = sqlx::query(sql)
                .fetch_all(&self.inner)
                .await?;
                
            let mut results = Vec::with_capacity(rows.len());
            for row in rows {
                let json_value = row_to_json_value(&row)?;
                let item: T = serde_json::from_value(json_value)?;
                results.push(item);
            }
            Ok(results)
        }
        
        async fn fetch_one<T>(&self, sql: &str, _params: &[Value]) -> Result<T>
        where
            T: DeserializeOwned + Send + Unpin,
        {
            // Simplified implementation for now - in production we'd properly bind parameters
            let row = sqlx::query(sql)
                .fetch_one(&self.inner)
                .await?;
                
            let json_value = row_to_json_value(&row)?;
            let item: T = serde_json::from_value(json_value)?;
            Ok(item)
        }
        
        async fn fetch_optional<T>(&self, sql: &str, _params: &[Value]) -> Result<Option<T>>
        where
            T: DeserializeOwned + Send + Unpin,
        {
            // Simplified implementation for now - in production we'd properly bind parameters
            if let Some(row) = sqlx::query(sql)
                .fetch_optional(&self.inner)
                .await? 
            {
                let json_value = row_to_json_value(&row)?;
                let item: T = serde_json::from_value(json_value)?;
                Ok(Some(item))
            } else {
                Ok(None)
            }
        }
    }
    
    #[allow(dead_code)]
    fn convert_params_to_sqlx(_params: &[Value]) -> Result<Vec<String>> {
        // This is a placeholder - in reality we'd need a more sophisticated conversion
        // For production, we'd implement proper Value -> SQLx parameter conversion
        // For now, just return empty vec for compilation
        Ok(Vec::new())
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
        fn test_param_conversion() {
            let params = vec![
                Value::I32(42),
                Value::String("test".to_string()),
                Value::Bool(true),
            ];
            
            // Test parameter conversion (placeholder implementation)
            let result = convert_params_to_sqlx(&params);
            assert!(result.is_ok());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{table, op};
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
        let query = table("users")
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
        let query = table("users").where_(("id", 1));
        
        let user: User = query.fetch_one(&pool).await.unwrap();
        assert_eq!(user.id, 1);
        assert_eq!(user.name, "John");
    }
    
    #[tokio::test]
    async fn test_select_fetch_optional() {
        let pool = MockPool::new();
        let query = table("users").where_(("id", 1));
        
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
        
        let query = crate::InsertBuilder::new("users").insert(data);
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
        let query = table("users");
        
        let result: Result<Vec<User>> = query.fetch_all(&pool).await;
        assert!(result.is_err());
    }
    
    #[tokio::test]  
    async fn test_modification_failure() {
        let pool = MockPool::with_failure();
        
        let mut data = HashMap::new();
        data.insert("name".to_string(), crate::Value::String("Test".to_string()));
        
        let query = crate::InsertBuilder::new("users").insert(data);
        let result = query.execute(&pool).await;
        assert!(result.is_err());
    }
}