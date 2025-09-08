use archibald::{delete, from, insert, op, update, QueryBuilder};
use archibald::{ColumnSelector, SortDirection};
use std::collections::HashMap;

fn main() {
    println!("=== Archibald Core - Basic Usage Examples ===\n");

    // SELECT with clean where syntax
    let select_query = from("users")
        .select(("id", "name", "email"))
        .where_(("age", op::GT, 18)) // Using op constants
        .where_(("status", "active")) // Defaults to EQ
        .and_where(("city", "LIKE", "%York%")) // Using string operators
        .limit(10)
        .offset(5);

    println!("1. Basic SELECT:");
    println!("   SQL: {}", select_query.to_sql().unwrap());
    println!("   Parameters: {:?}\n", select_query.parameters());

    // Subqueries - WHERE IN
    let subquery_in = from("users").select(("id", "name")).where_in(
        "id",
        from("orders")
            .select("user_id")
            .where_(("status", "completed"))
            .and_where(("created_at", op::GT, "2024-01-01")),
    );

    println!("2. Subquery (WHERE IN):");
    println!("   SQL: {}", subquery_in.to_sql().unwrap());

    // Subqueries - EXISTS
    let subquery_exists = from("users").select(("id", "name", "email")).where_exists(
        from("posts")
            .select("1")
            .and_where(("posts.author_id", "users.id"))
            .and_where(("posts.published", true)),
    );

    println!("   SQL: {}", subquery_exists.to_sql().unwrap());
    println!();

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

    println!("3. JOINs with Aggregation:");
    println!("   SQL: {}", join_query.to_sql().unwrap());
    println!();

    // INSERT
    let mut user_data = HashMap::new();
    user_data.insert("name".to_string(), "John Doe".into());
    user_data.insert("email".to_string(), "john@example.com".into());
    user_data.insert("age".to_string(), 30.into());

    let insert_query = insert("users").values(user_data);
    println!("4. INSERT:");
    println!("   SQL: {}", insert_query.to_sql().unwrap());
    println!("   Parameters: {:?}\n", insert_query.parameters());

    // UPDATE
    let mut updates = HashMap::new();
    updates.insert("email".to_string(), "newemail@example.com".into());
    updates.insert("last_login".to_string(), "2024-01-15".into());

    let update_query = update("users")
        .set(updates)
        .where_(("id", 123))
        .and_where(("active", true)); // Using and_where for clarity

    println!("5. UPDATE:");
    println!("   SQL: {}", update_query.to_sql().unwrap());
    println!("   Parameters: {:?}\n", update_query.parameters());

    // DELETE
    let delete_query =
        delete("users")
            .where_(("age", op::LT, 13))
            .or_where(("last_login", op::LT, "2020-01-01"));

    println!("6. DELETE:");
    println!("   SQL: {}", delete_query.to_sql().unwrap());
    println!("   Parameters: {:?}\n", delete_query.parameters());

    // Custom operators for advanced database features
    let postgres_fts_query = from("documents")
        .select(("title", "content"))
        .where_((
            "content",
            archibald::Operator::custom("@@"),
            "search & query",
        ))
        .limit(20);

    println!("7. Custom Operators (PostgreSQL Full-Text Search):");
    println!("   SQL: {}", postgres_fts_query.to_sql().unwrap());
    println!();

    // Complex WHERE conditions
    let complex_query = from("users")
        .where_(("age", op::GTE, 18)) // First condition
        .and_where(("status", "active")) // Explicit AND (same as where_)
        .or_where(("role", "admin")) // Explicit OR
        .and_where(("verified", true)) // Back to AND
        .select(("id", "name")); // Complete the query

    println!("8. Complex WHERE (AND/OR combinations):");
    println!("   SQL: {}", complex_query.to_sql().unwrap());
    println!("   Parameters: {:?}\n", complex_query.parameters());

    // Deferred validation example
    println!("9. Deferred Validation:");
    let invalid_query = from("users")
        .where_(("age", "INVALID_OPERATOR", 18))
        .select(("id", "name"));

    match invalid_query.to_sql() {
        Ok(sql) => println!("   Unexpected success: {}", sql),
        Err(e) => println!("   ✓ Caught invalid operator: {}", e),
    }

    println!("\n=== All examples completed successfully! ===");
}
