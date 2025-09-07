# Archibald ☁️

A type-safe, async SQL query builder for Rust, inspired by knex.js.

Named after [Archibald Query](https://en.wikipedia.org/wiki/Archibald_Query), the inventor of fluff, because your database deserves queries with character.

[![Crates.io](https://img.shields.io/crates/v/archibald.svg)](https://crates.io/crates/archibald)
[![Documentation](https://docs.rs/archibald/badge.svg)](https://docs.rs/archibald)

## ✨ Features

- **🔗 Fluent API**: Clean, chainable query builder inspired by knex.js
- **⚡ Async-first**: Built for `tokio` with async/await throughout
- **🏦 Transactions**: Full transaction support with savepoints and isolation levels
- **📊 Rich Queries**: JOINs, aggregations, GROUP BY, HAVING, ORDER BY, DISTINCT
- **🔍 Subqueries**: IN, NOT IN, EXISTS, NOT EXISTS, and SELECT subqueries
- **🎯 Parameter Binding**: Automatic SQL injection prevention
- **🗄️ Multi-Database**: PostgreSQL support (MySQL, SQLite planned)

## 🛡️ Compile-Time Safety for Dangerous Operations

Archibald is **strongly opinionated** about naked updates and deletes. UPDATE and DELETE operations require WHERE clauses at **compile time**:

```rust
// ❌ Won't compile - missing WHERE clause
update("users").set(data).execute(&pool).await?;  // Compile error!

// ❌ Won't compile - missing WHERE clause
delete("users").execute(&pool).await?;  // Compile error!

// ✅ Safe - both SET and WHERE required
update("users")
    .set(data)
    .where_(("id", 1))
    .execute(&pool).await?;  // ✅ Compiles!

// ✅ Explicit mass updates allowed - if you really mean it
update("users")
    .set(data)
    .where_((1, 1))  // Explicit "update everything" signal
    .execute(&pool).await?;
```

**Why this matters:** Many SQL data disasters come from missing WHERE clauses. Archibald makes it impossible to forget them.

## 🚀 Quick Start

Add to your `Cargo.toml`:

```toml
[dependencies]
archibald = { version = "0.1", features = ["postgres"] }
sqlx = { version = "0.7", features = ["runtime-tokio", "postgres"] }
tokio = { version = "1.0", features = ["macros", "rt-multi-thread"] }
serde = { version = "1.0", features = ["derive"] }
```

## 📖 Basic Usage

```rust
use archibald::{from, update, delete, insert, op, transaction};
use archibald::executor::postgres::PostgresPool;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct User {
    id: i32,
    name: String,
    email: String,
    age: i32,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Connect to database
    let pool = PostgresPool::new("postgresql://user:password@localhost/mydb").await?;
    
    // SELECT query
    let users: Vec<User> = from("users")
        .select(("id", "name", "email", "age"))
        .where_(("age", op::GT, 18))
        .and_where(("status", "active"))
        .limit(10)
        .fetch_all(&pool)
        .await?;
    
    println!("Found {} users", users.len());
    Ok(())
}
```

## 🔍 Query Examples

### SELECT with WHERE conditions
```rust
let adults = from("users")
    .select(("id", "name", "email"))
    .where_(("age", op::GTE, 18))           // age >= 18
    .and_where(("status", "active"))           // status = 'active' (defaults to EQ)
    .and_where(("name", "LIKE", "%john%"))     // name LIKE '%john%'
    .fetch_all(&pool)
    .await?;
```

### JOINs and aggregations
```rust
let user_stats = from("users")
    .select((
        "users.name",
        ColumnSelector::count().as_alias("post_count"),
        ColumnSelector::avg("posts.rating").as_alias("avg_rating")
    ))
    .inner_join("posts", "users.id", "posts.user_id")
    .where_(("users.active", true))
    .group_by("users.id, users.name")
    .having(("COUNT(*)", op::GT, 5))
    .order_by("avg_rating", SortDirection::Desc)
    .fetch_all(&pool)
    .await?;
```

### Subqueries
```rust
// WHERE IN subquery
let active_commenters = from("users")
    .select(("id", "name"))
    .where_in("id", 
        from("comments")
            .select("user_id")
            .where_(("created_at", op::GT, "2024-01-01"))
    )
    .fetch_all(&pool)
    .await?;

// EXISTS subquery
let users_with_orders = from("users")
    .select(("id", "name"))
    .where_exists(
        from("orders")
            .select("1")
            .where_(("orders.user_id", "users.id"))
    )
    .fetch_all(&pool)
    .await?;
```

### INSERT
```rust
use std::collections::HashMap;

let mut user_data = HashMap::new();
user_data.insert("name".to_string(), "Alice".into());
user_data.insert("email".to_string(), "alice@example.com".into());
user_data.insert("age".to_string(), 25.into());

let affected = insert("users")
    .values(user_data)
    .execute(&pool)
    .await?;

println!("Inserted {} rows", affected);
```

### UPDATE
```rust
let mut updates = HashMap::new();
updates.insert("email".to_string(), "newemail@example.com".into());
updates.insert("last_login".to_string(), "2024-01-15".into());

let affected = update("users")
    .set(updates)
    .where_(("id", 123))
    .and_where(("active", true))
    .execute(&pool)
    .await?;
```

### DELETE
```rust
let affected = delete("users")
    .where_(("last_login", op::LT, "2020-01-01"))
    .or_where(("status", "inactive"))
    .execute(&pool)
    .await?;
```

## 🏦 Transactions

Archibald provides full transaction support with automatic commit/rollback:

```rust
use archibald::transaction;

// Simple transaction with automatic commit/rollback
let result = transaction(&pool, |txn| async move {
    // Insert user
    let user_id = insert("users")
        .values(user_data)
        .execute_tx(txn)
        .await? as i32;
    
    // Create associated profile
    let mut profile_data = HashMap::new();
    profile_data.insert("user_id".to_string(), user_id.into());
    profile_data.insert("bio".to_string(), "Hello world!".into());
    
    insert("user_profiles")
        .values(profile_data)
        .execute_tx(txn)
        .await?;
        
    Ok::<i32, Error>(user_id)
}).await?;

println!("Created user with ID: {}", result);
```

### Manual transaction control
```rust
let mut txn = pool.begin_transaction().await?;

// Use savepoints for nested transaction logic
txn.savepoint("before_risky_operation").await?;

match risky_operation(&mut txn).await {
    Ok(_) => {
        txn.release_savepoint("before_risky_operation").await?;
        txn.commit().await?;
    }
    Err(_) => {
        txn.rollback_to_savepoint("before_risky_operation").await?;
        // Continue with transaction...
        txn.rollback().await?;
    }
}
```

### Transaction isolation levels
```rust
use archibald::IsolationLevel;

let txn = pool.begin_transaction_with_isolation(IsolationLevel::Serializable).await?;
// ... use transaction
txn.commit().await?;
```

## 🔧 Advanced Features

### Custom operators for database-specific features
```rust
// PostgreSQL full-text search
let documents = from("articles")
    .select(("id", "title"))
    .where_(("content", Operator::custom("@@"), "search & query"))
    .fetch_all(&pool)
    .await?;

// PostGIS distance queries
let nearby = from("locations")
    .select(("id", "name"))
    .where_(("coordinates", Operator::custom("<->"), point))
    .limit(10)
    .fetch_all(&pool)
    .await?;
```

### Deferred validation
```rust
// Build queries without Result handling
let query = from("users")
    .where_(("age", "INVALID_OPERATOR", 18))  // Stored, not validated yet
    .and_where(("name", "John"));

// Validation happens at SQL generation
match query.to_sql() {
    Ok(sql) => println!("SQL: {}", sql),
    Err(e) => println!("Invalid query: {}", e), // "Unknown operator 'INVALID_OPERATOR'"
}
```

## 🛡️ Parameter Binding & Safety

Archibald provides safety through:

1. **Automatic parameter binding** - All values are parameterized
2. **Compile Time Where clauses** - UPDATE / DELETE statements require where clauses at compile time
3. **Validated SQL generation** - Invalid queries fail at runtime, not in database

```rust
// ✅ Safe - parameters are automatically bound
let users = from("users")
    .where_(("name", user_input))        // Automatically parameterized as $1
    .and_where(("age", op::GT, min_age))    // Automatically parameterized as $2
    .fetch_all(&pool)
    .await?;

// ✅ Safe - generates: SELECT * FROM users WHERE name = $1 AND age > $2
// Parameters: ["some_user_input", 18]
```

## 🗄️ Database Support

| Database   | Status | Features |
|------------|--------|----------|
| PostgreSQL | ✅ Full | All features, parameter binding, transactions |
| MySQL      | 🔄 Planned | Coming soon |
| SQLite     | 🔄 Planned | Coming soon |

## 📚 Documentation

- [API Documentation](https://docs.rs/archibald)
- [Examples](./archibald/examples/)
- [Migration from knex.js](./MIGRATION.md) *(coming soon)*

## 🚧 Roadmap

- [x] Core query builder (SELECT, INSERT, UPDATE, DELETE)
- [x] JOINs, subqueries, aggregations
- [x] SQL parameter binding & injection prevention
- [x] Transaction support with savepoints
- [x] Deferred validation architecture
- [ ] Schema builder (CREATE TABLE, ALTER TABLE, etc.)
- [ ] Migration system
- [ ] MySQL and SQLite support
- [ ] Compile-time schema validation
- [ ] Query optimization and caching

## 📄 License

Licensed under the [LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT

## 🙏 Acknowledgments

- Inspired by [knex.js](https://knexjs.org/) - the excellent JavaScript query builder
- Built on [SQLx](https://github.com/launchbadge/sqlx) for database connectivity
- Powered by [Tokio](https://tokio.rs/) for async runtime
