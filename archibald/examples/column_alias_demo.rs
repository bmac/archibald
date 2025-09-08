use archibald::{col, from, ColumnSelector, QueryBuilder};

fn main() {
    println!("=== Column Alias Demo - Multiple Syntax Options ===\n");

    // Example 1: Using the new col() function (Most ergonomic!)
    let query1 = from("users")
        .select(col("user_id").as_alias("id"))
        .limit(10);

    println!("1. Using col() function (recommended):");
    println!("   Code: col(\"user_id\").as_alias(\"id\")");
    println!("   SQL: {}", query1.to_sql().unwrap());
    println!();

    // Example 2: Using ColumnSelector::column() method
    let query2 = from("employees")
        .select(ColumnSelector::column("employee_id").as_alias("id"));

    println!("2. Using ColumnSelector::column() method:");
    println!("   Code: ColumnSelector::column(\"employee_id\").as_alias(\"id\")");
    println!("   SQL: {}", query2.to_sql().unwrap());
    println!();

    // Example 3: Mix of different approaches
    let query3 = from("orders")
        .select((
            col("customer_id").as_alias("cust_id"),
            "order_date", // Regular column
            ColumnSelector::sum("total_amount").as_alias("total_sales"),
        ))
        .group_by("customer_id");

    println!("3. Mixed approaches - col() + regular columns + aggregates:");
    println!("   SQL: {}", query3.to_sql().unwrap());
    println!();

    // Example 4: Vector of aliased columns (for many aliases)
    let query4 = from("users")
        .select(vec![
            col("users.id").as_alias("user_id"),
            col("users.name").as_alias("full_name"),
            col("profiles.bio").as_alias("biography"),
        ])
        .inner_join("profiles", "users.id", "profiles.user_id");

    println!("4. Multiple column aliases with col():");
    println!("   SQL: {}", query4.to_sql().unwrap());
    println!();

    // Example 5: Before and after comparison
    println!("5. Syntax comparison:");
    println!("   ❌ Verbose:  ColumnSelector::Column {{ name: \"col\".to_string(), alias: None }}.as_alias(\"alias\")");
    println!("   ✅ Method:   ColumnSelector::column(\"col\").as_alias(\"alias\")");
    println!("   🚀 Function: col(\"col\").as_alias(\"alias\")");
    println!();

    println!("=== Demo completed! ===");
    println!("Column aliases allow you to rename columns in the result set.");
    println!("The col() function provides the most ergonomic syntax!");
}