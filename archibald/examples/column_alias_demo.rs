use archibald::{from, ColumnSelector, QueryBuilder};

fn main() {
    println!("=== Column Alias Demo ===\n");

    // Example 1: Simple column alias
    let query1 = from("users")
        .select(ColumnSelector::Column {
            name: "user_id".to_string(),
            alias: None,
        }.as_alias("id"))
        .limit(10);

    println!("1. Simple column alias:");
    println!("   SQL: {}", query1.to_sql().unwrap());
    println!();

    // Example 2: Mix of column alias and regular column using supported tuple pattern
    let query2 = from("employees")
        .select((
            "department", // Regular column without alias
            ColumnSelector::Column {
                name: "first_name".to_string(),
                alias: None,
            }.as_alias("fname"),
        ));

    println!("2. Column alias with regular column:");
    println!("   SQL: {}", query2.to_sql().unwrap());
    println!();

    // Example 3: Mix of column alias and aggregate alias using supported pattern
    let query3 = from("orders")
        .select((
            ColumnSelector::Column {
                name: "customer_id".to_string(),
                alias: None,
            }.as_alias("cust_id"),
            "order_date", // Regular column
            ColumnSelector::sum("total_amount").as_alias("total_sales"),
        ))
        .group_by("customer_id");

    println!("3. Mixed column and aggregate aliases:");
    println!("   SQL: {}", query3.to_sql().unwrap());
    println!();

    // Example 4: Using vector for multiple column aliases
    let query4 = from("users")
        .select(vec![
            ColumnSelector::Column {
                name: "users.id".to_string(),
                alias: None,
            }.as_alias("user_id"),
            ColumnSelector::Column {
                name: "users.name".to_string(),
                alias: None,
            }.as_alias("full_name"),
            ColumnSelector::Column {
                name: "profiles.bio".to_string(),
                alias: None,
            }.as_alias("biography"),
        ])
        .inner_join("profiles", "users.id", "profiles.user_id");

    println!("4. Multiple column aliases using Vec:");
    println!("   SQL: {}", query4.to_sql().unwrap());
    println!();

    println!("=== Demo completed! ===");
    println!("Column aliases allow you to rename columns in the result set,");
    println!("making the output more user-friendly or compatible with your application's naming conventions.");
    println!("\nUsage: ColumnSelector::Column {{ name: \"column_name\".to_string(), alias: None }}.as_alias(\"alias_name\")");
}