use archibald_core::{table, op, InsertBuilder, UpdateBuilder, DeleteBuilder};
use archibald_core::{ConnectionPool, ExecutableQuery, ExecutableModification, Transaction, TransactionalPool, IsolationLevel, transaction};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct User {
    id: i32,
    name: String,
    email: String,
    balance: i32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct Account {
    id: i32,
    user_id: i32,
    balance: i32,
}

// Mock connection pool for demonstration (replace with real PostgresPool in production)
#[derive(Clone)]
struct MockPool;

impl ConnectionPool for MockPool {
    type Connection = ();
    
    async fn acquire(&self) -> archibald_core::Result<Self::Connection> {
        Ok(())
    }
    
    async fn execute(&self, sql: &str, _params: &[archibald_core::Value]) -> archibald_core::Result<u64> {
        println!("   EXECUTE: {}", sql);
        Ok(1) // Simulate 1 affected row
    }
    
    async fn fetch_all<T>(&self, sql: &str, _params: &[archibald_core::Value]) -> archibald_core::Result<Vec<T>>
    where
        T: serde::de::DeserializeOwned + Send + Unpin,
    {
        println!("   FETCH_ALL: {}", sql);
        Ok(Vec::new())
    }
    
    async fn fetch_one<T>(&self, sql: &str, _params: &[archibald_core::Value]) -> archibald_core::Result<T>
    where
        T: serde::de::DeserializeOwned + Send + Unpin,
    {
        println!("   FETCH_ONE: {}", sql);
        
        if std::any::type_name::<T>().contains("User") {
            let user_json = serde_json::json!({"id": 1, "name": "Alice", "email": "alice@example.com", "balance": 1000});
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
        println!("   FETCH_OPTIONAL: {}", sql);
        Ok(None)
    }
}

// Mock transaction for demonstration
struct MockTransaction;

impl Transaction for MockTransaction {
    async fn execute(&mut self, sql: &str, _params: &[archibald_core::Value]) -> archibald_core::Result<u64> {
        println!("   TXN EXECUTE: {}", sql);
        Ok(1)
    }
    
    async fn fetch_all<T>(&mut self, sql: &str, _params: &[archibald_core::Value]) -> archibald_core::Result<Vec<T>>
    where
        T: serde::de::DeserializeOwned + Send + Unpin,
    {
        println!("   TXN FETCH_ALL: {}", sql);
        Ok(Vec::new())
    }
    
    async fn fetch_one<T>(&mut self, sql: &str, _params: &[archibald_core::Value]) -> archibald_core::Result<T>
    where
        T: serde::de::DeserializeOwned + Send + Unpin,
    {
        println!("   TXN FETCH_ONE: {}", sql);
        
        if std::any::type_name::<T>().contains("User") {
            let user_json = serde_json::json!({"id": 1, "name": "Alice", "email": "alice@example.com", "balance": 1000});
            let user: T = serde_json::from_value(user_json)?;
            Ok(user)
        } else {
            return Err(archibald_core::Error::sql_generation("No mock data for this type"));
        }
    }
    
    async fn fetch_optional<T>(&mut self, sql: &str, _params: &[archibald_core::Value]) -> archibald_core::Result<Option<T>>
    where
        T: serde::de::DeserializeOwned + Send + Unpin,
    {
        println!("   TXN FETCH_OPTIONAL: {}", sql);
        Ok(None)
    }
    
    async fn commit(self) -> archibald_core::Result<()> {
        println!("   TXN COMMIT");
        Ok(())
    }
    
    async fn rollback(self) -> archibald_core::Result<()> {
        println!("   TXN ROLLBACK");
        Ok(())
    }
    
    async fn savepoint(&mut self, name: &str) -> archibald_core::Result<()> {
        println!("   TXN SAVEPOINT: {}", name);
        Ok(())
    }
    
    async fn rollback_to_savepoint(&mut self, name: &str) -> archibald_core::Result<()> {
        println!("   TXN ROLLBACK TO SAVEPOINT: {}", name);
        Ok(())
    }
    
    async fn release_savepoint(&mut self, name: &str) -> archibald_core::Result<()> {
        println!("   TXN RELEASE SAVEPOINT: {}", name);
        Ok(())
    }
}

impl TransactionalPool for MockPool {
    type Transaction = MockTransaction;
    
    async fn begin_transaction(&self) -> archibald_core::Result<Self::Transaction> {
        println!("   BEGIN TRANSACTION");
        Ok(MockTransaction)
    }
    
    async fn begin_transaction_with_isolation(&self, isolation: IsolationLevel) -> archibald_core::Result<Self::Transaction> {
        println!("   BEGIN TRANSACTION ({})", isolation.to_sql());
        Ok(MockTransaction)
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let pool = MockPool;
    
    println!("=== Archibald Transaction Examples ===\n");
    
    // Example 1: Simple transaction with automatic commit/rollback
    println!("1. Simple Transaction (automatic commit/rollback):");
    let user_id = transaction(&pool, |txn| async move {
        println!("   Creating user and profile atomically...");
        
        // Insert user
        let mut user_data = HashMap::new();
        user_data.insert("name".to_string(), "Bob Smith".into());
        user_data.insert("email".to_string(), "bob@example.com".into());
        user_data.insert("balance".to_string(), 0.into());
        
        let user_id = InsertBuilder::new("users")
            .insert(user_data)
            .execute_tx(txn)
            .await? as i32;
        
        // Create user profile
        let mut profile_data = HashMap::new();
        profile_data.insert("user_id".to_string(), user_id.into());
        profile_data.insert("bio".to_string(), "New user".into());
        profile_data.insert("avatar".to_string(), "default.png".into());
        
        InsertBuilder::new("user_profiles")
            .insert(profile_data)
            .execute_tx(txn)
            .await?;
        
        Ok::<i32, archibald_core::Error>(user_id)
    }).await?;
    
    println!("   ✓ User created with ID: {}\\n", user_id);
    
    // Example 2: Transaction with error handling (automatic rollback)
    println!("2. Transaction with Error (automatic rollback):");
    let result = transaction(&pool, |_txn| async move {
        println!("   Simulating an operation that fails...");
        Err::<(), archibald_core::Error>(archibald_core::Error::sql_generation("Simulated business logic error"))
    }).await;
    
    match result {
        Ok(_) => println!("   Unexpected success"),
        Err(e) => println!("   ✓ Transaction rolled back due to error: {}\\n", e),
    }
    
    // Example 3: Manual transaction control with savepoints
    println!("3. Manual Transaction with Savepoints:");
    let mut txn = pool.begin_transaction().await?;
    
    // Transfer money between accounts
    println!("   Transferring $100 from Account 1 to Account 2...");
    
    // Debit from account 1
    let mut debit_updates = HashMap::new();
    debit_updates.insert("balance".to_string(), "balance - 100".into()); // Note: In real use, you'd fetch current balance first
    
    UpdateBuilder::new("accounts")
        .set(debit_updates)
        .where_(("id", 1))
        .execute_tx(&mut txn)
        .await?;
    
    // Create savepoint before crediting
    txn.savepoint("before_credit").await?;
    
    // Credit to account 2 
    let mut credit_updates = HashMap::new();
    credit_updates.insert("balance".to_string(), "balance + 100".into());
    
    let credit_result = UpdateBuilder::new("accounts")
        .set(credit_updates)
        .where_(("id", 2))
        .execute_tx(&mut txn)
        .await;
    
    match credit_result {
        Ok(_) => {
            println!("   ✓ Credit successful, releasing savepoint");
            txn.release_savepoint("before_credit").await?;
            txn.commit().await?;
        }
        Err(_) => {
            println!("   ✗ Credit failed, rolling back to savepoint");
            txn.rollback_to_savepoint("before_credit").await?;
            txn.rollback().await?;
        }
    }
    
    // Example 4: Transaction isolation levels
    println!("\\n4. Transaction with Isolation Level:");
    let mut serializable_txn = pool.begin_transaction_with_isolation(IsolationLevel::Serializable).await?;
    
    // Perform operations requiring serializable isolation
    let _users: Vec<User> = table("users")
        .select(("id", "name", "email", "balance"))
        .where_(("balance", op::GT, 1000))
        .fetch_all_tx(&mut serializable_txn)
        .await?;
    
    // Update based on the read (classic read-modify-write)
    UpdateBuilder::new("users")
        .set({
            let mut updates = HashMap::new();
            updates.insert("vip_status".to_string(), true.into());
            updates
        })
        .where_(("balance", op::GT, 1000))
        .execute_tx(&mut serializable_txn)
        .await?;
    
    serializable_txn.commit().await?;
    println!("   ✓ Serializable transaction completed\\n");
    
    // Example 5: Nested business logic with multiple savepoints
    println!("5. Complex Business Logic with Multiple Savepoints:");
    let mut complex_txn = pool.begin_transaction().await?;
    
    // Savepoint 1: User creation
    complex_txn.savepoint("user_creation").await?;
    
    let mut user_data = HashMap::new();
    user_data.insert("name".to_string(), "Complex User".into());
    user_data.insert("email".to_string(), "complex@example.com".into());
    
    InsertBuilder::new("users")
        .insert(user_data)
        .execute_tx(&mut complex_txn)
        .await?;
    
    // Savepoint 2: Account setup
    complex_txn.savepoint("account_setup").await?;
    
    let mut account_data = HashMap::new();
    account_data.insert("user_id".to_string(), 123.into());
    account_data.insert("account_type".to_string(), "premium".into());
    account_data.insert("balance".to_string(), 5000.into());
    
    InsertBuilder::new("accounts")
        .insert(account_data)
        .execute_tx(&mut complex_txn)
        .await?;
    
    // Simulate complex validation that might fail
    let validation_passed = true; // In real code, this would be actual validation
    
    if validation_passed {
        println!("   ✓ All validations passed");
        complex_txn.release_savepoint("account_setup").await?;
        complex_txn.release_savepoint("user_creation").await?;
        complex_txn.commit().await?;
    } else {
        println!("   ✗ Validation failed, rolling back account setup");
        complex_txn.rollback_to_savepoint("account_setup").await?;
        // Could continue with user creation only, or rollback everything
        complex_txn.rollback().await?;
    }
    
    println!("\\n=== All transaction examples completed! ===");
    println!("\\nIn production, replace MockPool with archibald_core::executor::postgres::PostgresPool");
    
    Ok(())
}