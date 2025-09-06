# Archibald - Rust Query Builder Plan

## Overview

Archibald is a Rust query builder inspired by the JavaScript knex.js library. It aims to be 75% faithful to the knex API while leveraging Rust's type system, ownership model, and performance characteristics to create a more robust and performant query builder.

## Core Philosophy

- **Type Safety**: Leverage Rust's type system to prevent SQL injection and catch errors at compile time
- **Zero-cost Abstractions**: Minimize runtime overhead while maintaining ergonomic API
- **Memory Safety**: Use Rust's ownership model to eliminate common database interaction bugs
- **Async First**: Built from the ground up for async/await with `tokio` integration
- **Compile-time SQL Validation**: Where possible, validate SQL structure at compile time

## Key Rust Adaptations from Knex

### 1. Type System Improvements

**Knex JavaScript:**
```javascript
const users = await knex('users')
  .select('id', 'name', 'email')
  .where('age', '>', 18);
```

**Archibald Rust:**
```rust
use archibald::op;

let users: Vec<User> = archibald::table("users")
    .select(("id", "name", "email"))
    .where(("age", op::GT, 18))
    .fetch(&pool)
    .await?;
```

**Improvements:**
- **Strong Typing**: Return typed structs instead of generic objects
- **Compile-time Column Validation**: Detect invalid column names at compile time (with proc macros)
- **SQL Injection Prevention**: All parameters are automatically parameterized
- **SQL-like Syntax**: `where(("age", op::GT, 18))` maintains familiar syntax while preventing invalid operators

### 2. Error Handling

**Knex JavaScript:**
```javascript
try {
  const result = await knex('users').insert({name: 'John'});
} catch (error) {
  // Runtime error handling
}
```

**Archibald Rust:**
```rust
let result = archibald::table("users")
    .insert(NewUser { name: "John".to_string() })
    .execute(&pool)
    .await?; // Compile-time error handling with Result<T, E>
```

**Improvements:**
- **Result Type**: All operations return `Result<T, ArchibaldError>` for explicit error handling
- **Error Categorization**: Specific error types for different failure modes (connection, constraint, etc.)
- **No Runtime Surprises**: Errors must be handled explicitly

### 3. Ownership and Borrowing

**Knex JavaScript:**
```javascript
const query = knex('users').select('*');
const filtered = query.where('active', true); // Mutates original query
```

**Archibald Rust:**
```rust
let query = archibald::table("users").select_all();
let filtered = query.where(("active", true)); // Returns new query, original remains unchanged
```

**Improvements:**
- **Immutable by Default**: Query builders are immutable; methods return new instances
- **No Hidden Mutations**: Clear ownership semantics prevent accidental query modification
- **Zero-copy When Possible**: Use `Cow<'_, str>` for table/column names to avoid unnecessary allocations

### 4. Async/Await Integration

**Knex JavaScript:**
```javascript
const result = await knex('users').insert({name: 'John'});
```

**Archibald Rust:**
```rust
let result = archibald::table("users")
    .insert(NewUser { name: "John".to_string() })
    .execute(&pool)
    .await?;
```

**Improvements:**
- **Native Async**: Built with `async`/`await` from the ground up
- **Tokio Integration**: First-class support for tokio runtime
- **Connection Pool Efficiency**: Optimized for async connection pooling

### 5. Memory Safety and Performance

**Knex JavaScript:**
```javascript
const query = knex.raw('SELECT * FROM users WHERE id = ?', [userId]);
```

**Archibald Rust:**
```rust
let query = archibald::raw("SELECT * FROM users WHERE id = $1", &[&user_id]);
// or using the type-safe builder:
let query = archibald::table("users")
    .select_all()
    .where_eq("id", user_id);
```

**Improvements:**
- **Stack Allocation**: Query builders use stack allocation where possible
- **Zero-copy String Operations**: Minimize allocations for static SQL fragments
- **Connection Pool Optimization**: Rust's ownership enables more efficient connection management

### 6. Macro System for Compile-time Validation

**Knex JavaScript:**
```javascript
// Runtime error if column doesn't exist
const users = await knex('users').select('non_existent_column');
```

**Archibald Rust:**
```rust
// Compile-time error with proc macros (future feature)
archibald_table! {
    users {
        id: i32,
        name: String,
        email: String,
    }
}

let users = archibald::users()
    .select((users::id, users::name)) // Compile-time column validation
    .fetch(&pool)
    .await?;
```

**Improvements:**
- **Schema Definition Macros**: Define table structure at compile time
- **Column Validation**: Invalid column references fail compilation
- **IntelliSense Support**: Full IDE autocomplete for table and column names

## API Improvements for Rust

### 1. Method Name Clarification

| Knex (JavaScript) | Archibald (Rust) | Reason |
|------------------|------------------|---------|
| `.where('age', '>', 18)` | `.where(("age", op::GT, 18))` | Maintains SQL-like syntax with type safety |
| `.where('name', 'like', '%john%')` | `.where(("name", op::LIKE, "%john%"))` | Familiar syntax, restricted operators |
| `.where('id', 'in', [1,2,3])` | `.where(("id", op::IN, &[1,2,3]))` | Type-safe array handling |
| `.where('age', 18)` (implied =) | `.where(("age", 18))` | Shorthand for equality, defaults to EQ |
| `.join('posts', 'users.id', 'posts.user_id')` | `.inner_join::<posts>("users.id", "posts.user_id")` | Explicit join type |

### 2. Type-safe Aggregations

**Knex JavaScript:**
```javascript
const count = await knex('users').count('*');
const avg = await knex('users').avg('age');
```

**Archibald Rust:**
```rust
let count: i64 = archibald::table("users")
    .count()
    .fetch_one(&pool)
    .await?;

let avg: Option<f64> = archibald::table("users")
    .avg("age")
    .fetch_one(&pool)
    .await?;
```

### 3. Transaction API

**Knex JavaScript:**
```javascript
await knex.transaction(async (trx) => {
  await trx('users').insert({name: 'John'});
  await trx('posts').insert({user_id: 1, title: 'Hello'});
});
```

**Archibald Rust:**
```rust
archibald::transaction(&pool, |trx| async move {
    archibald::table("users")
        .insert(NewUser { name: "John".to_string() })
        .execute(&trx)
        .await?;
    
    archibald::table("posts")
        .insert(NewPost { user_id: 1, title: "Hello".to_string() })
        .execute(&trx)
        .await?;
    
    Ok(())
}).await?;
```

### 4. Raw Query Safety

**Knex JavaScript:**
```javascript
const result = await knex.raw('SELECT * FROM users WHERE id = ?', [userId]);
```

**Archibald Rust:**
```rust
// Type-safe raw queries
let result: Vec<User> = archibald::raw_query("SELECT * FROM users WHERE id = $1")
    .bind(user_id)
    .fetch(&pool)
    .await?;

// Or with macro for compile-time validation
let result = archibald_raw!(
    "SELECT * FROM users WHERE id = $1",
    user_id
).fetch::<User>(&pool).await?;
```

## Dependencies

Archibald follows a minimal dependency approach, keeping the core lightweight while providing optional features:

### Core Dependencies (Required)
```toml
[dependencies]
# Core runtime
tokio = { version = "1.0", features = ["rt", "macros"] }
futures = "0.3"

# Serialization  
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# Database foundation
sqlx = { version = "0.7", features = ["runtime-tokio", "any"], default-features = false }

# Error handling
thiserror = "1.0"
```

### Optional Dependencies (Feature-gated)
```toml
# Database-specific types
uuid = { version = "1.0", optional = true, features = ["v4", "serde"] }
chrono = { version = "0.4", optional = true, default-features = false, features = ["serde"] }
rust_decimal = { version = "1.0", optional = true, features = ["serde"] }

[features]
default = ["postgres"]

# Database drivers
postgres = ["sqlx/postgres"]
mysql = ["sqlx/mysql"] 
sqlite = ["sqlx/sqlite"]
mssql = ["sqlx/mssql"]

# Optional type support
uuid-support = ["uuid"]
datetime-support = ["chrono"]
decimal-support = ["rust_decimal"]
all-types = ["uuid-support", "datetime-support", "decimal-support"]
```

**Design Principles:**
- **Minimal Core**: Only 6 essential dependencies for the base functionality
- **Feature Gates**: Database drivers and type support are opt-in
- **SQLx Foundation**: Leverage proven async database connectivity and connection pooling
- **Pay for What You Use**: Users only include the features they need

**Why SQLx?**
- Proven async-first database abstraction
- Multi-database support (PostgreSQL, MySQL, SQLite, MSSQL)
- Built-in connection pooling and prepared statements
- Type-safe query compilation (we can build on this)
- Handles low-level database complexity

## Implementation Plan

### Phase 1: Core Foundation (Weeks 1-3)
1. **Project Setup**
   - Create Cargo workspace structure with dependency configuration
   - Set up CI/CD with GitHub Actions for multiple database testing
   - Create basic error types and result handling with thiserror

2. **Core Query Builder Structure**
   - Define `QueryBuilder` trait and basic implementations
   - Implement method chaining pattern with immutable builders
   - Create `SelectBuilder`, `InsertBuilder`, `UpdateBuilder`, `DeleteBuilder`
   - Basic SQL generation without database-specific optimizations

3. **Connection and Pool Management**
   - Abstract connection pool interface over SQLx pools
   - Integration with `sqlx::Pool` for connection management
   - Basic async execution framework with tokio

### Phase 2: Select Queries (Weeks 4-6)
1. **Basic SELECT Operations**
   - `select()`, `select_all()`, `from()` methods
   - WHERE clause builders (`where_eq`, `where_gt`, `where_lt`, etc.)
   - Logical operators (`and()`, `or()`, `not()`)
   - Result deserialization with `serde`

2. **Advanced SELECT Features**  
   - JOIN operations (inner, left, right, full outer)
   - ORDER BY and GROUP BY
   - LIMIT and OFFSET
   - DISTINCT operations
   - Subqueries support

3. **Aggregation Functions**
   - COUNT, SUM, AVG, MIN, MAX
   - HAVING clauses
   - Type-safe aggregation results

### Phase 3: Modification Queries (Weeks 7-8)
1. **INSERT Operations**
   - Single and batch inserts
   - RETURNING clause support  
   - ON CONFLICT handling (PostgreSQL/SQLite)
   - Type-safe insertion with struct deserialization

2. **UPDATE and DELETE Operations**
   - Basic UPDATE with WHERE conditions
   - Batch updates
   - DELETE operations with safety checks
   - Conditional modifications

### Phase 4: Schema Management (Weeks 9-10)
1. **Schema Builder**
   - Table creation and modification
   - Column definitions with types
   - Index management
   - Foreign key constraints

2. **Migration System**
   - Migration file management
   - Up/down migration support
   - Schema versioning
   - Rollback capabilities

### Phase 5: Advanced Features (Weeks 11-12)
1. **Transaction Support**
   - Nested transaction handling
   - Savepoints
   - Transaction isolation levels
   - Deadlock detection and retry

2. **Database Dialect Support**
   - PostgreSQL-specific features
   - MySQL optimizations
   - SQLite compatibility
   - Pluggable dialect system

### Phase 6: Performance and Safety (Weeks 13-14)
1. **Compile-time Validation**
   - Proc macro for schema definition
   - Compile-time column validation
   - SQL injection prevention at type level

2. **Performance Optimizations**
   - Query plan caching
   - Connection pool tuning
   - Memory allocation optimization
   - Benchmark suite

### Phase 7: Documentation and Ecosystem (Weeks 15-16)
1. **Documentation**
   - API documentation with examples
   - Migration guide from knex.js
   - Best practices guide
   - Performance tuning guide

2. **Ecosystem Integration**
   - Serde integration examples
   - Testing utilities
   - Example applications

## Crate Structure

```
archibald/
├── archibald-core/           # Core query building logic
├── archibald-macros/         # Compile-time validation macros
├── archibald-migrate/        # Migration system  
├── archibald-postgres/       # PostgreSQL dialect
├── archibald-mysql/          # MySQL dialect
├── archibald-sqlite/         # SQLite dialect
├── archibald-derive/         # Derive macros for structs
└── archibald/                # Main crate (re-exports)
```

## Success Metrics

1. **API Compatibility**: 75% of knex.js patterns have direct Rust equivalents
2. **Type Safety**: Zero SQL injection vulnerabilities possible through normal API usage
3. **Performance**: Comparable or better performance than knex.js in benchmarks
4. **Documentation**: Comprehensive docs with migration examples from knex.js
5. **Ecosystem**: Integration examples with major Rust web frameworks

## Migration Path from Knex.js

The library will include comprehensive migration documentation showing equivalent patterns:

| Feature | Knex.js | Archibald |
|---------|---------|-----------|
| Basic select | `knex('users').select('*')` | `archibald::table("users").select_all()` |
| Where clause | `knex('users').where('age', '>', 18)` | `archibald::table("users").where_(("age", op::GT, 18))` |
| Where equality | `knex('users').where('age', 18)` | `archibald::table("users").where_(("age", 18))` |
| Joins | `knex('users').join('posts', 'users.id', 'posts.user_id')` | `archibald::table("users").inner_join::<posts>("users.id", "posts.user_id")` |
| Transactions | `knex.transaction(trx => { ... })` | `archibald::transaction(&pool, \|trx\| async move { ... })` |

## Trait-Based Where Conditions API

The elegant `where((...))` syntax is achieved through a trait-based approach with type-safe operators:

```rust
// Type-safe operator struct
#[derive(Debug, Clone, PartialEq)]
pub struct Operator(&'static str);

impl Operator {
    pub const GT: Self = Operator(">");
    pub const LT: Self = Operator("<");
    pub const EQ: Self = Operator("=");
    pub const NEQ: Self = Operator("!=");
    pub const GTE: Self = Operator(">=");
    pub const LTE: Self = Operator("<=");
    pub const LIKE: Self = Operator("LIKE");
    pub const ILIKE: Self = Operator("ILIKE");
    pub const IN: Self = Operator("IN");
    pub const NOT_IN: Self = Operator("NOT IN");
    pub const IS_NULL: Self = Operator("IS NULL");
    pub const IS_NOT_NULL: Self = Operator("IS NOT NULL");
    
    // Escape hatch for custom operators
    pub const fn custom(op: &'static str) -> Self {
        Operator(op)
    }
}

// Trait to convert various types to operators
pub trait IntoOperator {
    fn into_operator(self) -> Operator;
}

impl IntoOperator for Operator {
    fn into_operator(self) -> Operator {
        self
    }
}

// Allow string literals for common SQL operators (with validation)
impl IntoOperator for &str {
    fn into_operator(self) -> Operator {
        match self {
            ">" => Operator::GT,
            "<" => Operator::LT,
            "=" => Operator::EQ,
            "!=" => Operator::NEQ,
            ">=" => Operator::GTE,
            "<=" => Operator::LTE,
            "LIKE" | "like" => Operator::LIKE,
            "ILIKE" | "ilike" => Operator::ILIKE,
            "IN" | "in" => Operator::IN,
            "NOT IN" | "not in" => Operator::NOT_IN,
            _ => panic!(
                "Unknown operator '{}'. Use op::{} constants or Operator::custom(\"{}\") for custom operators.", 
                self, self.to_uppercase().replace(" ", "_"), self
            )
        }
    }
}

// Condition trait for where clauses
pub trait IntoCondition {
    fn into_condition(self) -> (String, Operator, Value);
}

// For shorthand equality: where(("age", 18))
impl<T> IntoCondition for (&str, T) 
where 
    T: Into<Value>
{
    fn into_condition(self) -> (String, Operator, Value) {
        (self.0.to_string(), Operator::EQ, self.1.into())
    }
}

// For explicit operators: where(("age", op::GT, 18)) or where(("age", ">", 18))
impl<T, O> IntoCondition for (&str, O, T) 
where 
    T: Into<Value>,
    O: IntoOperator
{
    fn into_condition(self) -> (String, Operator, Value) {
        (self.0.to_string(), self.1.into_operator(), self.2.into())
    }
}

// The where method accepts anything implementing IntoCondition
impl QueryBuilder {
    pub fn where<C>(self, condition: C) -> Self 
    where 
        C: IntoCondition
    {
        let (column, operator, value) = condition.into_condition();
        // Add to where conditions...
        self
    }
}
```

**Usage Examples:**
```rust
use archibald::op;

archibald::table("users")
    .where_(("age", op::GT, 18))           // Using op constants
    .where_(("score", ">", 100))           // Using string literals (validated)
    .where_(("name", "John"))              // Defaults to EQ
    .where_(("status", op::IN, &["active", "pending"]))  // IN clause
    .where_(("email", "LIKE", "%@gmail.com"))            // LIKE pattern

// Custom operators for advanced use cases:
archibald::table("documents")
    .where_(("content", Operator::custom("@@"), "search term"))  // PostgreSQL full-text search
    .where_(("location", Operator::custom("<->"), point))        // PostGIS distance operator

// Chained where_() calls are implicitly AND'd together
// For OR conditions:
archibald::table("users")
    .where_(("age", ">", 18))
    .or_where(("status", "admin"))        // Explicit OR
    .and_where(("active", true))          // Explicit AND (same as .where_())
```

**Benefits:**
- **Type Safety**: Only valid operators compile or run (panics on invalid strings)
- **Familiar Syntax**: `where_(("age", ">", 18))` reads like SQL and knex
- **Flexible**: Supports constants (`op::GT`), strings (`">"`), and shorthand equality
- **Extensible**: `Operator::custom()` escape hatch for database-specific operators
- **Performance**: Zero-cost operator constants, validated strings panic early
- **Clear Errors**: Helpful panic messages suggest correct alternatives

## WHERE Clause Method Design Decision

**Method Name Choice: `where_` vs `where`**

We chose to use `where_` (with trailing underscore) instead of Rust's raw identifier syntax `r#where` for the following reasons:

**Why not `r#where`:**
- Difficult to type and remember the `r#` prefix
- Poor autocomplete experience (doesn't show up when typing "where")  
- Unusual syntax that most Rust developers aren't familiar with
- Creates cognitive overhead when reading code

**Why `where_`:**
- **Autocomplete friendly**: Starts typing "where" and gets suggestions immediately
- **Clear intent**: Obviously maps to SQL WHERE clause
- **Conventional**: Many Rust libraries use trailing underscores for reserved keywords
- **Readable**: No special syntax or prefixes needed

**Method Relationship:**
- `.where_(condition)` - Primary method for WHERE conditions (implicit AND)
- `.and_where(condition)` - Explicit AND condition (identical to `.where_()`)
- `.or_where(condition)` - Explicit OR condition

**Usage Examples:**
```rust
// All these are equivalent for AND conditions:
table("users").where_(("age", op::GT, 18)).where_(("status", "active"))
table("users").where_(("age", op::GT, 18)).and_where(("status", "active"))

// Mixed AND/OR conditions:
table("users")
    .where_(("age", op::GT, 18))     // First condition
    .and_where(("status", "active")) // Explicit AND  
    .or_where(("role", "admin"))     // OR condition
    .and_where(("verified", true))   // Back to AND
```

This approach prioritizes developer ergonomics while maintaining clear SQL semantics.

## Deferred Validation Architecture Design Decision

**Problem**: Original design had inconsistent validation approaches - operators would panic immediately during query building, while subqueries attempted to return Results throughout the chain, creating a poor user experience.

**Solution**: Implement deferred validation architecture where all validation happens at `to_sql()` time.

**Key Design Principles:**
1. **Clean Fluent API**: No `Result` handling required during query building
2. **User-Friendly Error Handling**: Library provides "escape valve" via `to_sql()` instead of panicking
3. **Consistent Architecture**: Both operators and subqueries use same deferred validation pattern

**Implementation Details:**

### Operator System (src/operator.rs)
```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Operator {
    /// Known valid operator
    Known(&'static str),
    /// Unknown operator (validated at to_sql() time)
    Unknown(String),
}

impl Operator {
    /// Validate that this operator is recognized (used at to_sql() time)
    pub fn validate(&self) -> Result<()> {
        match self {
            Operator::Known(_) => Ok(()),
            Operator::Unknown(op) => {
                Err(Error::invalid_query(format!(
                    "Unknown operator '{}'. Use Operator::{} constants or Operator::custom(\\\"{}\\\") for custom operators.", 
                    op, op.to_uppercase().replace(" ", "_").replace("!", "N"), op
                )))
            }
        }
    }
}
```

### Query Builder Validation (src/builder.rs)
All query builders validate operators and subqueries at `to_sql()` time:

```rust
impl QueryBuilder for SelectBuilder {
    fn to_sql(&self) -> Result<String> {
        // Validate all operators before generating SQL
        for condition in &self.where_conditions {
            condition.operator.validate()?;
        }
        
        for condition in &self.having_conditions {
            condition.operator.validate()?;
        }
        
        for condition in &self.subquery_conditions {
            condition.operator.validate()?;
            // Validate subquery recursively
            condition.subquery.query.to_sql()?;
        }
        
        for join_clause in &self.join_clauses {
            for condition in &join_clause.on_conditions {
                condition.operator.validate()?;
            }
        }
        
        // Generate SQL after all validation passes...
    }
}
```

**Benefits:**
- **Ergonomic API**: Users can chain methods fluently without dealing with Results
- **Clear Error Point**: All validation errors surface at the single `to_sql()` call
- **Helpful Error Messages**: Invalid operators provide suggestions for correct usage
- **Consistent Experience**: Same pattern for operators, subqueries, and future validation

**Example Usage:**
```rust
// Building queries never panics or requires Result handling
let query = table("users")
    .where_(("age", "INVALID_OPERATOR", 18))  // Stores as Unknown variant
    .where_in("id", subquery);                // Defers subquery validation

// All validation happens here with clear error messages
match query.to_sql() {
    Ok(sql) => println!("SQL: {}", sql),
    Err(e) => println!("Invalid query: {}", e), // "Unknown operator 'INVALID_OPERATOR'. Use..."
}
```

This architecture prioritizes developer experience while maintaining type safety and providing clear error handling.

---

This plan creates a Rust query builder that feels familiar to knex.js users while leveraging Rust's unique advantages for better safety, performance, and developer experience.
