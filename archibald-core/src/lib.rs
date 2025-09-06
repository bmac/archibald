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
pub use builder::{QueryBuilder, SelectBuilder, InsertBuilder, UpdateBuilder, DeleteBuilder, JoinType, SortDirection};
pub use value::Value;
pub use executor::{ConnectionPool, ExecutableQuery, ExecutableModification};

/// Create a new query builder for the given table
pub fn table(name: &str) -> SelectBuilder {
    SelectBuilder::new(name)
}