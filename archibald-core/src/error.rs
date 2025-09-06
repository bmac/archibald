//! Error types for Archibald

use thiserror::Error;

/// The main error type for Archibald operations
#[derive(Error, Debug)]
pub enum Error {
    /// Database connection or execution error
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    
    /// SQL generation error
    #[error("SQL generation error: {message}")]
    SqlGeneration { message: String },
    
    /// Invalid query configuration
    #[error("Invalid query: {message}")]
    InvalidQuery { message: String },
    
    /// Serialization error
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    
    /// Column not found error
    #[error("Column '{column}' not found in table '{table}'")]
    ColumnNotFound { table: String, column: String },
    
    /// Table not found error
    #[error("Table '{table}' not found")]
    TableNotFound { table: String },
}

/// Convenience Result type for Archibald operations
pub type Result<T> = std::result::Result<T, Error>;

impl Error {
    /// Create a new SQL generation error
    pub fn sql_generation(message: impl Into<String>) -> Self {
        Self::SqlGeneration {
            message: message.into(),
        }
    }
    
    /// Create a new invalid query error
    pub fn invalid_query(message: impl Into<String>) -> Self {
        Self::InvalidQuery {
            message: message.into(),
        }
    }
    
    /// Create a new column not found error
    pub fn column_not_found(table: impl Into<String>, column: impl Into<String>) -> Self {
        Self::ColumnNotFound {
            table: table.into(),
            column: column.into(),
        }
    }
    
    /// Create a new table not found error
    pub fn table_not_found(table: impl Into<String>) -> Self {
        Self::TableNotFound {
            table: table.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_error_creation() {
        let err = Error::sql_generation("Invalid SELECT");
        assert!(matches!(err, Error::SqlGeneration { .. }));
        assert_eq!(err.to_string(), "SQL generation error: Invalid SELECT");
    }
    
    #[test]
    fn test_invalid_query_error() {
        let err = Error::invalid_query("Missing WHERE clause");
        assert!(matches!(err, Error::InvalidQuery { .. }));
        assert_eq!(err.to_string(), "Invalid query: Missing WHERE clause");
    }
    
    #[test]
    fn test_column_not_found_error() {
        let err = Error::column_not_found("users", "invalid_column");
        assert!(matches!(err, Error::ColumnNotFound { .. }));
        assert_eq!(err.to_string(), "Column 'invalid_column' not found in table 'users'");
    }
    
    #[test]
    fn test_table_not_found_error() {
        let err = Error::table_not_found("non_existent_table");
        assert!(matches!(err, Error::TableNotFound { .. }));
        assert_eq!(err.to_string(), "Table 'non_existent_table' not found");
    }
}