//! Archibald Core - A type-safe SQL query builder inspired by knex.js
//!
//! This crate provides the core functionality for building SQL queries in a
//! fluent, immutable, and type-safe manner.

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
    UpdateBuilder,
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
pub fn update(name: &str) -> UpdateBuilder {
    builder::UpdateBuilder::new(name)
}

/// Create a new DELETE query builder for the given table
pub fn delete(name: &str) -> DeleteBuilderInitial {
    builder::DeleteBuilderInitial::new(name)
}

/// Create a new INSERT query builder for the given table
pub fn insert(name: &str) -> InsertBuilderInitial {
    builder::InsertBuilderInitial::new(name)
}
