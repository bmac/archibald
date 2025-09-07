//! Archibald Core - A type-safe SQL query builder inspired by knex.js
//! 
//! This crate provides the core functionality for building SQL queries in a 
//! fluent, immutable, and type-safe manner.

pub mod error;
pub mod operator;
pub mod builder;
pub mod value;
pub mod executor;

// Re-export main types
pub use error::{Error, Result};
pub use operator::{Operator, IntoOperator, op};
pub use builder::{QueryBuilder, SelectBuilder, InsertBuilder, UpdateBuilder, DeleteBuilder, JoinType, SortDirection, ColumnSelector, AggregateFunction, Subquery};
pub use value::Value;
pub use executor::{
    ConnectionPool, ExecutableQuery, ExecutableModification, 
    Transaction, TransactionalPool, IsolationLevel, transaction
};

/// Create a new SELECT query builder for the given table
pub fn from(name: &str) -> SelectBuilder {
    SelectBuilder::new(name)
}

/// Create a new UPDATE query builder for the given table
pub fn update(name: &str) -> UpdateBuilder {
    UpdateBuilder::new(name)
}

/// Create a new DELETE query builder for the given table
pub fn delete(name: &str) -> DeleteBuilder {
    DeleteBuilder::new(name)
}

/// Create a new INSERT query builder for the given table
pub fn insert(name: &str) -> InsertBuilder {
    InsertBuilder::new(name)
}