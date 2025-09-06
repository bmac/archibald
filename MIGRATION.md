# Migration Guide: From knex.js to Archibald

This guide helps you migrate from [knex.js](https://knexjs.org/) to Archibald, highlighting the similarities and differences between the two query builders.

## Overview

Archibald is heavily inspired by knex.js and maintains similar fluent API patterns while leveraging Rust's type safety and async capabilities. Most knex.js concepts translate directly to Archibald with minor syntax adjustments.

## Basic Setup

### knex.js
```javascript
const knex = require('knex')({
  client: 'postgresql',
  connection: {
    host: '127.0.0.1',
    port: 5432,
    user: 'your_database_user',
    password: 'your_database_password',
    database: 'myapp_test'
  }
});
```

### Archibald
```rust
use archibald_core::executor::postgres::PostgresPool;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let pool = PostgresPool::new(
        "postgresql://your_database_user:your_database_password@127.0.0.1:5432/myapp_test"
    ).await?;
    
    // Use pool for queries...
    Ok(())
}
```

## SELECT Queries

### Basic SELECT

**knex.js:**
```javascript
const users = await knex('users')
  .select('id', 'name', 'email')
  .where('age', '>', 18)
  .where('status', 'active')
  .limit(10);
```

**Archibald:**
```rust
use archibald_core::{from, op};

let users: Vec<User> = from("users")
    .select(("id", "name", "email"))
    .where_(("age", op::GT, 18))
    .where_(("status", "active"))
    .limit(10)
    .fetch_all(&pool)
    .await?;
```

### WHERE conditions with different operators

**knex.js:**
```javascript
const users = await knex('users')
  .where('age', '>=', 21)
  .where('name', 'like', '%john%')
  .where('verified', true);
```

**Archibald:**
```rust
let users: Vec<User> = from("users")
    .where_(("age", op::GTE, 21))
    .where_(("name", "LIKE", "%john%"))
    .where_(("verified", true))
    .fetch_all(&pool)
    .await?;
```

### OR conditions

**knex.js:**
```javascript
const users = await knex('users')
  .where('role', 'admin')
  .orWhere('status', 'premium');
```

**Archibald:**
```rust
let users: Vec<User> = from("users")
    .where_(("role", "admin"))
    .or_where(("status", "premium"))
    .fetch_all(&pool)
    .await?;
```

## JOINs

### Inner JOIN

**knex.js:**
```javascript
const results = await knex('users')
  .select('users.name', 'posts.title')
  .innerJoin('posts', 'users.id', 'posts.user_id')
  .where('users.active', true);
```

**Archibald:**
```rust
let results = from("users")
    .select(("users.name", "posts.title"))
    .inner_join("posts", "users.id", "posts.user_id")
    .where_(("users.active", true))
    .fetch_all(&pool)
    .await?;
```

### LEFT JOIN with aggregation

**knex.js:**
```javascript
const userStats = await knex('users')
  .select('users.name')
  .count('posts.id as post_count')
  .avg('posts.rating as avg_rating')
  .leftJoin('posts', 'users.id', 'posts.user_id')
  .groupBy('users.id', 'users.name')
  .having('count', '>', 5);
```

**Archibald:**
```rust
use archibald_core::ColumnSelector;

let user_stats = from("users")
    .select((
        "users.name",
        ColumnSelector::count().as_alias("post_count"),
        ColumnSelector::avg("posts.rating").as_alias("avg_rating")
    ))
    .left_join("posts", "users.id", "posts.user_id")
    .group_by("users.id, users.name")
    .having(("count", op::GT, 5))
    .fetch_all(&pool)
    .await?;
```

## Subqueries

### WHERE IN subquery

**knex.js:**
```javascript
const activeUsers = await knex('users')
  .select('id', 'name')
  .whereIn('id', 
    knex('orders')
      .select('user_id')
      .where('status', 'completed')
      .where('created_at', '>', '2024-01-01')
  );
```

**Archibald:**
```rust
let active_users: Vec<User> = from("users")
    .select(("id", "name"))
    .where_in("id",
        from("orders")
            .select("user_id")
            .where_(("status", "completed"))
            .where_(("created_at", op::GT, "2024-01-01"))
    )
    .fetch_all(&pool)
    .await?;
```

### EXISTS subquery

**knex.js:**
```javascript
const usersWithPosts = await knex('users')
  .select('id', 'name')
  .whereExists(
    knex('posts')
      .select(1)
      .whereRaw('posts.user_id = users.id')
      .where('posts.published', true)
  );
```

**Archibald:**
```rust
let users_with_posts: Vec<User> = from("users")
    .select(("id", "name"))
    .where_exists(
        from("posts")
            .select("1")
            .where_(("posts.user_id", "users.id"))
            .where_(("posts.published", true))
    )
    .fetch_all(&pool)
    .await?;
```

## INSERT Operations

### Single INSERT

**knex.js:**
```javascript
const [userId] = await knex('users')
  .insert({
    name: 'John Doe',
    email: 'john@example.com',
    age: 30
  })
  .returning('id');
```

**Archibald:**
```rust
use std::collections::HashMap;
use archibald_core::InsertBuilder;

let mut user_data = HashMap::new();
user_data.insert("name".to_string(), "John Doe".into());
user_data.insert("email".to_string(), "john@example.com".into());
user_data.insert("age".to_string(), 30.into());

let affected = InsertBuilder::new("users")
    .insert(user_data)
    .execute(&pool)
    .await?;
```

## UPDATE Operations

**knex.js:**
```javascript
const affected = await knex('users')
  .where('id', 123)
  .update({
    email: 'newemail@example.com',
    updated_at: knex.fn.now()
  });
```

**Archibald:**
```rust
use archibald_core::UpdateBuilder;

let mut updates = HashMap::new();
updates.insert("email".to_string(), "newemail@example.com".into());
updates.insert("updated_at".to_string(), "NOW()".into());

let affected = UpdateBuilder::new("users")
    .set(updates)
    .where_(("id", 123))
    .execute(&pool)
    .await?;
```

## DELETE Operations

**knex.js:**
```javascript
const affected = await knex('users')
  .where('last_login', '<', '2020-01-01')
  .orWhere('status', 'inactive')
  .del();
```

**Archibald:**
```rust
use archibald_core::DeleteBuilder;

let affected = DeleteBuilder::new("users")
    .where_(("last_login", op::LT, "2020-01-01"))
    .or_where(("status", "inactive"))
    .execute(&pool)
    .await?;
```

## Transactions

### Simple transaction

**knex.js:**
```javascript
const result = await knex.transaction(async (trx) => {
  const [userId] = await trx('users')
    .insert({ name: 'Alice', email: 'alice@example.com' })
    .returning('id');
    
  await trx('user_profiles')
    .insert({ user_id: userId, bio: 'Hello world!' });
    
  return userId;
});
```

**Archibald:**
```rust
use archibald_core::transaction;

let user_id = transaction(&pool, |txn| async move {
    let user_id = InsertBuilder::new("users")
        .insert({
            let mut data = HashMap::new();
            data.insert("name".to_string(), "Alice".into());
            data.insert("email".to_string(), "alice@example.com".into());
            data
        })
        .execute_tx(txn)
        .await? as i32;
    
    let mut profile_data = HashMap::new();
    profile_data.insert("user_id".to_string(), user_id.into());
    profile_data.insert("bio".to_string(), "Hello world!".into());
    
    InsertBuilder::new("user_profiles")
        .insert(profile_data)
        .execute_tx(txn)
        .await?;
        
    Ok::<i32, archibald_core::Error>(user_id)
}).await?;
```

### Manual transaction control

**knex.js:**
```javascript
const trx = await knex.transaction();

try {
  await trx.savepoint('sp1');
  
  await trx('accounts')
    .where('id', 1)
    .update({ balance: knex.raw('balance - 100') });
    
  await trx('accounts')
    .where('id', 2)
    .update({ balance: knex.raw('balance + 100') });
    
  await trx.commit();
} catch (error) {
  await trx.rollback();
  throw error;
}
```

**Archibald:**
```rust
let mut txn = pool.begin_transaction().await?;

txn.savepoint("sp1").await?;

UpdateBuilder::new("accounts")
    .set({
        let mut updates = HashMap::new();
        updates.insert("balance".to_string(), "balance - 100".into());
        updates
    })
    .where_(("id", 1))
    .execute_tx(&mut txn)
    .await?;

UpdateBuilder::new("accounts")
    .set({
        let mut updates = HashMap::new();
        updates.insert("balance".to_string(), "balance + 100".into());
        updates
    })
    .where_(("id", 2))
    .execute_tx(&mut txn)
    .await?;

txn.commit().await?;
```

## Key Differences

### 1. Type Safety
- **knex.js**: Runtime errors for SQL issues
- **Archibald**: Compile-time and structured runtime error handling

### 2. Async Handling
- **knex.js**: Promise-based with async/await
- **Archibald**: Native async/await with Result types for error handling

### 3. Parameter Binding
- **knex.js**: Automatic parameter binding
- **Archibald**: Automatic parameter binding with type-safe Value enum

### 4. Method Names
- **knex.js**: `where()`, `orWhere()`
- **Archibald**: `where_()`, `or_where()` (Rust naming conventions)

### 5. Column Selection
- **knex.js**: `select('col1', 'col2')` or `select(['col1', 'col2'])`
- **Archibald**: `select(("col1", "col2"))` (tuple syntax)

### 6. Value Types
- **knex.js**: JavaScript native types
- **Archibald**: Rust types that implement `Into<Value>`

### 7. Error Handling
- **knex.js**: Try/catch or .catch()
- **Archibald**: Result<T, E> with ? operator

## Migration Tips

1. **Start with type definitions**: Define your Rust structs with Serde derive macros
2. **Update connection setup**: Replace knex configuration with Archibald pool creation
3. **Convert method names**: Change camelCase to snake_case (where → where_, orWhere → or_where)
4. **Add type annotations**: Specify return types for fetch operations
5. **Handle Results**: Use ? operator or match statements for error handling
6. **Update column selectors**: Use tuple syntax for multiple columns
7. **Convert raw SQL**: Use custom operators or raw query methods when needed

## Performance Considerations

- **knex.js**: Single-threaded JavaScript runtime
- **Archibald**: Multi-threaded Rust with connection pooling and async I/O

## Next Steps

After migrating your basic queries, consider leveraging Archibald's additional features:

- **Type safety**: Use strongly typed structs for query results
- **Advanced transactions**: Utilize savepoints and isolation levels
- **Custom operators**: Implement database-specific functionality
- **Connection pooling**: Optimize for high-concurrency scenarios

For more examples and detailed API documentation, see:
- [Examples directory](./archibald-core/examples/)
- [API Documentation](https://docs.rs/archibald-core)