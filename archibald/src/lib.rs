//! Archibald - A type-safe SQL query builder for Rust inspired by knex.js
//!
//! Archibald provides a fluent, immutable, and type-safe API for building SQL queries
//! with compile-time guarantees and an intuitive builder pattern.

pub mod builder;
pub mod error;
pub mod executor;
pub mod operator;
pub mod value;

// Re-export main types
pub use builder::common::{
    AggregateFunction, IntoCondition, JoinType, QueryBuilder, SortDirection, WhereCondition,
    WhereConnector,
};
pub use builder::select::{ColumnSelector, SelectBuilderComplete, SelectBuilderInitial, Subquery};
pub use builder::{
    DeleteBuilderComplete, DeleteBuilderInitial, InsertBuilderComplete, InsertBuilderInitial,
    UpdateBuilderComplete, UpdateBuilderInitial, UpdateBuilderWithSet,
};
pub use error::{Error, Result};
pub use executor::{
    transaction, ConnectionPool, ExecutableModification, ExecutableQuery, IsolationLevel,
    Transaction, TransactionalPool,
};
pub use operator::{op, IntoOperator, Operator};
pub use value::Value;

/// Create a new SELECT query builder for the given table
pub fn from(name: &str) -> SelectBuilderInitial {
    builder::select::SelectBuilderInitial::new(name)
}

/// Create a new UPDATE query builder for the given table
pub fn update(name: &str) -> UpdateBuilderInitial {
    builder::UpdateBuilderInitial::new(name)
}

/// Create a new DELETE query builder for the given table
pub fn delete(name: &str) -> DeleteBuilderInitial {
    builder::DeleteBuilderInitial::new(name)
}

/// Create a new INSERT query builder for the given table
pub fn insert(name: &str) -> InsertBuilderInitial {
    builder::InsertBuilderInitial::new(name)
}

/// Create a column selector for aliasing
/// 
/// # Examples
/// 
/// ```
/// use archibald::col;
/// 
/// // Create a column that can be aliased
/// let aliased_column = col("order_id").as_alias("id");
/// ```
pub fn col(name: &str) -> ColumnSelector {
    ColumnSelector::column(name)
}
