use archibald_core::{table, op, InsertBuilder, UpdateBuilder, DeleteBuilder, QueryBuilder};
use std::collections::HashMap;

fn main() {
    // SELECT with clean where syntax
    let select_query = table("users")
        .select(("id", "name", "email"))
        .where_(("age", op::GT, 18))        // Using op constants
        .where_(("status", "active"))       // Defaults to EQ
        .where_(("city", "LIKE", "%York%")) // Using string operators
        .limit(10)
        .offset(5);
    
    println!("SELECT SQL: {}", select_query.to_sql().unwrap());
    
    // INSERT
    let mut user_data = HashMap::new();
    user_data.insert("name".to_string(), "John Doe".into());
    user_data.insert("email".to_string(), "john@example.com".into());
    user_data.insert("age".to_string(), 30.into());
    
    let insert_query = InsertBuilder::new("users").insert(user_data);
    println!("INSERT SQL: {}", insert_query.to_sql().unwrap());
    
    // UPDATE
    let mut updates = HashMap::new();
    updates.insert("email".to_string(), "newemail@example.com".into());
    updates.insert("last_login".to_string(), "2024-01-15".into());
    
    let update_query = UpdateBuilder::new("users")
        .set(updates)
        .where_(("id", 123))
        .and_where(("active", true));  // Using and_where for clarity
    
    println!("UPDATE SQL: {}", update_query.to_sql().unwrap());
    
    // DELETE
    let delete_query = DeleteBuilder::new("users")
        .where_(("age", op::LT, 13))
        .or_where(("last_login", op::LT, "2020-01-01"));
        
    println!("DELETE SQL: {}", delete_query.to_sql().unwrap());
    
    // Custom operators for advanced database features
    let postgres_fts_query = table("documents")
        .select(("title", "content"))
        .where_(("content", archibald_core::Operator::custom("@@"), "search query"))
        .limit(20);
    
    // Demonstrating and_where vs or_where vs where_ 
    let complex_query = table("users")
        .where_(("age", op::GTE, 18))     // First condition
        .and_where(("status", "active"))  // Explicit AND (same as where_)
        .or_where(("role", "admin"))      // Explicit OR
        .and_where(("verified", true));   // Back to AND
    
    println!("Complex query: {}", complex_query.to_sql().unwrap());
        
    println!("PostgreSQL FTS SQL: {}", postgres_fts_query.to_sql().unwrap());
}