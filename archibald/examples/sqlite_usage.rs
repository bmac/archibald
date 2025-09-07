use archibald::{delete, from, insert, op, update, QueryBuilder};
use archibald::{ColumnSelector, SortDirection};
use std::collections::HashMap;

// Note: Import SqlitePool if you need to actually connect to a database
// #[cfg(feature = "sqlite")]
// use archibald::executor::sqlite::SqlitePool;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(feature = "sqlite")]
    {
        println!("=== Archibald SQLite - Usage Example ===\n");

        // Note: This example demonstrates SQL generation only
        // To actually connect to SQLite: let pool = SqlitePool::new("sqlite:example.db").await?;

        // SELECT with clean where syntax
        let select_query = from("users")
            .select(("id", "name", "email"))
            .where_(("age", op::GT, 18)) // Using op constants
            .where_(("status", "active")) // Defaults to EQ
            .where_(("city", "LIKE", "%York%")) // Using string operators
            .limit(10)
            .offset(5);

        println!("1. Basic SELECT:");
        println!("   SQL: {}", select_query.to_sql()?);
        println!("   Parameters: {:?}\n", select_query.parameters());

        // JOINs with aggregations
        let join_query = from("users")
            .select((
                "users.name",
                ColumnSelector::count().as_alias("post_count"),
                ColumnSelector::avg("posts.rating").as_alias("avg_rating"),
            ))
            .inner_join("posts", "users.id", "posts.author_id")
            .where_(("users.active", true))
            .group_by("users.id, users.name")
            .having(("COUNT(*)", op::GT, 5))
            .order_by("avg_rating", SortDirection::Desc)
            .limit(20);

        println!("2. JOINs with Aggregation:");
        println!("   SQL: {}", join_query.to_sql()?);
        println!();

        // INSERT
        let mut user_data = HashMap::new();
        user_data.insert("name".to_string(), "John Doe".into());
        user_data.insert("email".to_string(), "john@example.com".into());
        user_data.insert("age".to_string(), 30.into());

        let insert_query = insert("users").values(user_data);
        println!("3. INSERT:");
        println!("   SQL: {}", insert_query.to_sql()?);
        println!("   Parameters: {:?}\n", insert_query.parameters());

        // UPDATE
        let mut updates = HashMap::new();
        updates.insert("email".to_string(), "newemail@example.com".into());
        updates.insert("last_login".to_string(), "2024-01-15".into());

        let update_query = update("users")
            .set(updates)
            .where_(("id", 123))
            .and_where(("active", true));

        println!("4. UPDATE:");
        println!("   SQL: {}", update_query.to_sql()?);
        println!("   Parameters: {:?}\n", update_query.parameters());

        // DELETE
        let delete_query =
            delete("users")
                .where_(("age", op::LT, 13))
                .or_where(("last_login", op::LT, "2020-01-01"));

        println!("5. DELETE:");
        println!("   SQL: {}", delete_query.to_sql()?);
        println!("   Parameters: {:?}\n", delete_query.parameters());

        // SQLite-specific features
        println!("6. SQLite-specific considerations:");
        println!("   - JSON values are serialized as TEXT");
        println!("   - Arrays are serialized as JSON strings");
        println!("   - Limited isolation levels (maps to PRAGMA settings)");
        println!("   - Single writer, multiple readers model");
        println!();

        // Subqueries - EXISTS
        let subquery_exists = from("users").select(("id", "name", "email")).where_exists(
            from("posts")
                .select("1")
                .where_(("posts.author_id", "users.id"))
                .where_(("posts.published", true)),
        );

        println!("7. Subquery (WHERE EXISTS):");
        println!("   SQL: {}", subquery_exists.to_sql()?);
        println!();

        // Example with JSON data (SQLite stores as TEXT)
        let mut json_data = HashMap::new();
        json_data.insert("name".to_string(), "Jane Doe".into());
        json_data.insert("metadata".to_string(), serde_json::json!({"role": "admin", "permissions": ["read", "write"]}).into());

        let insert_json_query = insert("users").values(json_data);
        println!("8. INSERT with JSON (SQLite stores as TEXT):");
        println!("   SQL: {}", insert_json_query.to_sql()?);
        println!("   Parameters: {:?}\n", insert_json_query.parameters());

        println!("=== All examples completed successfully! ===");
        println!("Note: This example demonstrates SQL generation only.");
        println!("To execute queries against a real SQLite database, use the executor methods:");
        println!("  - query.fetch_all(&pool).await?");
        println!("  - query.execute(&pool).await?");
    }

    #[cfg(not(feature = "sqlite"))]
    {
        println!("SQLite feature not enabled. Please run with: cargo run --example sqlite_usage --features sqlite");
    }

    Ok(())
}