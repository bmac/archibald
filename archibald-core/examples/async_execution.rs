use archibald_core::{from, insert, update, delete, op};
use archibald_core::{ConnectionPool, ExecutableQuery, ExecutableModification};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct User {
    id: i32,
    name: String,
    email: String,
    age: i32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct NewUser {
    name: String,
    email: String,
    age: i32,
}

// Mock connection pool for demonstration
#[derive(Clone)]
struct MockPool;

impl ConnectionPool for MockPool {
    type Connection = ();
    
    async fn acquire(&self) -> archibald_core::Result<Self::Connection> {
        Ok(())
    }
    
    async fn execute(&self, sql: &str, _params: &[archibald_core::Value]) -> archibald_core::Result<u64> {
        println!("EXECUTE: {}", sql);
        Ok(1) // Simulate 1 affected row
    }
    
    async fn fetch_all<T>(&self, sql: &str, _params: &[archibald_core::Value]) -> archibald_core::Result<Vec<T>>
    where
        T: serde::de::DeserializeOwned + Send + Unpin,
    {
        println!("FETCH_ALL: {}", sql);
        
        // Return mock data for User type
        if std::any::type_name::<T>().contains("User") {
            let users_json = serde_json::json!([
                {"id": 1, "name": "Alice", "email": "alice@example.com", "age": 25},
                {"id": 2, "name": "Bob", "email": "bob@example.com", "age": 30},
                {"id": 3, "name": "Charlie", "email": "charlie@example.com", "age": 35}
            ]);
            let users: Vec<T> = serde_json::from_value(users_json)?;
            Ok(users)
        } else {
            Ok(Vec::new())
        }
    }
    
    async fn fetch_one<T>(&self, sql: &str, _params: &[archibald_core::Value]) -> archibald_core::Result<T>
    where
        T: serde::de::DeserializeOwned + Send + Unpin,
    {
        println!("FETCH_ONE: {}", sql);
        
        if std::any::type_name::<T>().contains("User") {
            let user_json = serde_json::json!({"id": 1, "name": "Alice", "email": "alice@example.com", "age": 25});
            let user: T = serde_json::from_value(user_json)?;
            Ok(user)
        } else {
            return Err(archibald_core::Error::sql_generation("No mock data for this type"));
        }
    }
    
    async fn fetch_optional<T>(&self, sql: &str, _params: &[archibald_core::Value]) -> archibald_core::Result<Option<T>>
    where
        T: serde::de::DeserializeOwned + Send + Unpin,
    {
        println!("FETCH_OPTIONAL: {}", sql);
        
        if std::any::type_name::<T>().contains("User") {
            let user_json = serde_json::json!({"id": 1, "name": "Alice", "email": "alice@example.com", "age": 25});
            let user: T = serde_json::from_value(user_json)?;
            Ok(Some(user))
        } else {
            Ok(None)
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let pool = MockPool;
    
    println!("=== Archibald Async Execution Demo ===\n");
    
    // SELECT queries with execution
    println!("1. Fetch all users:");
    let users: Vec<User> = from("users")
        .select(("id", "name", "email", "age"))
        .where_(("age", op::GT, 18))
        .fetch_all(&pool)
        .await?;
    
    println!("Found {} users:", users.len());
    for user in &users {
        println!("  - {}: {} ({})", user.id, user.name, user.email);
    }
    
    println!("\n2. Fetch single user:");
    let user: User = from("users")
        .where_(("id", 1))
        .fetch_one(&pool)
        .await?;
    println!("User: {} - {}", user.name, user.email);
    
    println!("\n3. Fetch optional user:");
    let maybe_user: Option<User> = from("users")
        .where_(("id", 999))
        .fetch_optional(&pool)
        .await?;
    match maybe_user {
        Some(user) => println!("Found user: {}", user.name),
        None => println!("No user found with id 999"),
    }
    
    // INSERT query
    println!("\n4. Insert new user:");
    let mut new_user_data = HashMap::new();
    new_user_data.insert("name".to_string(), "David".into());
    new_user_data.insert("email".to_string(), "david@example.com".into());
    new_user_data.insert("age".to_string(), 28.into());
    
    let affected = insert("users")
        .values(new_user_data)
        .execute(&pool)
        .await?;
    println!("Inserted {} row(s)", affected);
    
    // UPDATE query
    println!("\n5. Update user:");
    let mut updates = HashMap::new();
    updates.insert("email".to_string(), "newemail@example.com".into());
    updates.insert("age".to_string(), 29.into());
    
    let affected = update("users")
        .set(updates)
        .where_(("id", 1))
        .and_where(("name", "Alice"))
        .execute(&pool)
        .await?;
    println!("Updated {} row(s)", affected);
    
    // DELETE query
    println!("\n6. Delete inactive users:");
    let affected = delete("users")
        .where_(("last_login", op::LT, "2020-01-01"))
        .or_where(("status", "inactive"))
        .execute(&pool)
        .await?;
    println!("Deleted {} row(s)", affected);
    
    // Complex query with multiple conditions
    println!("\n7. Complex query:");
    let active_users: Vec<User> = from("users")
        .select(("id", "name", "email", "age"))
        .where_(("age", op::GTE, 21))
        .and_where(("status", "active"))
        .or_where(("role", "admin"))
        .and_where(("verified", true))
        .limit(10)
        .offset(0)
        .fetch_all(&pool)
        .await?;
    
    println!("Found {} active users", active_users.len());
    
    println!("\n8. Subquery Examples:");
    
    // WHERE IN subquery
    let active_users: Vec<User> = from("users")
        .select(("id", "name", "email", "age"))
        .where_in("id",
            from("orders")
                .select("user_id") 
                .where_(("status", "completed"))
                .where_(("created_at", op::GT, "2024-01-01"))
        )
        .fetch_all(&pool)
        .await?;
    
    println!("Found {} users with completed orders", active_users.len());
    
    // EXISTS subquery
    let users_with_posts: Vec<User> = from("users") 
        .select(("id", "name", "email", "age"))
        .where_exists(
            from("posts")
                .select("1")
                .where_(("posts.author_id", "users.id"))
                .where_(("posts.published", true))
        )
        .fetch_all(&pool)
        .await?;
        
    println!("Found {} users with published posts", users_with_posts.len());
    
    println!("\n9. Transaction Example:");
    
    // Simple transaction (Note: MockPool doesn't implement TransactionalPool)
    // In production, you would use PostgresPool which supports transactions:
    /*
    use archibald_core::{transaction, executor::postgres::PostgresPool};
    
    let pg_pool = PostgresPool::new("postgresql://...").await?;
    
    let user_id = transaction(&pg_pool, |txn| async move {
        // Insert user
        let user_id = insert("users")
            .values(user_data)
            .execute_tx(txn)
            .await? as i32;
        
        // Insert user profile
        let mut profile_data = HashMap::new();
        profile_data.insert("user_id".to_string(), user_id.into());
        profile_data.insert("bio".to_string(), "New user".into());
        
        insert("user_profiles")
            .values(profile_data)
            .execute_tx(txn)
            .await?;
            
        Ok::<i32, Error>(user_id)
    }).await?;
    
    println!("Created user with ID: {}", user_id);
    */
    
    println!("Transactions require a real database pool (see transactions.rs example)");

    println!("\n=== Demo completed successfully! ===");
    println!("\nTo run with real PostgreSQL:");
    println!("1. Add PostgreSQL connection string");
    println!("2. Replace MockPool with PostgresPool");
    println!("3. Enable the 'postgres' feature flag");
    
    Ok(())
}