//! SQL operator types and conversions

use std::fmt::{self, Display};

/// Type-safe SQL operator
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Operator {
    /// Known valid operator
    Known(&'static str),
    /// Unknown operator (validated at to_sql() time)
    Unknown(String),
}

impl Operator {
    pub const GT: Self = Operator::Known(">");
    pub const LT: Self = Operator::Known("<");
    pub const EQ: Self = Operator::Known("=");
    pub const NEQ: Self = Operator::Known("!=");
    pub const GTE: Self = Operator::Known(">=");
    pub const LTE: Self = Operator::Known("<=");
    pub const LIKE: Self = Operator::Known("LIKE");
    pub const ILIKE: Self = Operator::Known("ILIKE");
    pub const IN: Self = Operator::Known("IN");
    pub const NOT_IN: Self = Operator::Known("NOT IN");
    pub const IS_NULL: Self = Operator::Known("IS NULL");
    pub const IS_NOT_NULL: Self = Operator::Known("IS NOT NULL");
    pub const EXISTS: Self = Operator::Known("EXISTS");
    pub const NOT_EXISTS: Self = Operator::Known("NOT EXISTS");
    
    /// Create a custom operator for database-specific operations
    /// 
    /// # Examples
    /// ```
    /// use archibald_core::Operator;
    /// 
    /// // PostgreSQL full-text search
    /// let fts_op = Operator::custom("@@");
    /// 
    /// // PostGIS distance operator
    /// let distance_op = Operator::custom("<->");
    /// ```
    pub const fn custom(op: &'static str) -> Self {
        Operator::Known(op)
    }
    
    /// Get the string representation of the operator
    pub fn as_str(&self) -> &str {
        match self {
            Operator::Known(op) => op,
            Operator::Unknown(op) => op,
        }
    }
    
    /// Validate that this operator is recognized (used at to_sql() time)
    pub fn validate(&self) -> crate::Result<()> {
        match self {
            Operator::Known(_) => Ok(()),
            Operator::Unknown(op) => {
                Err(crate::Error::invalid_query(format!(
                    "Unknown operator '{}'. Use Operator::{} constants or Operator::custom(\"{}\") for custom operators.", 
                    op,
                    op.to_uppercase().replace(" ", "_").replace("!", "N"), 
                    op
                )))
            }
        }
    }
}

impl Display for Operator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Trait for types that can be converted to SQL operators
pub trait IntoOperator {
    fn into_operator(self) -> Operator;
}

impl IntoOperator for Operator {
    fn into_operator(self) -> Operator {
        self
    }
}

/// Allow string literals for common SQL operators (validation deferred to to_sql())
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
            "IS NULL" | "is null" => Operator::IS_NULL,
            "IS NOT NULL" | "is not null" => Operator::IS_NOT_NULL,
            "EXISTS" | "exists" => Operator::EXISTS,
            "NOT EXISTS" | "not exists" => Operator::NOT_EXISTS,
            // Store unknown operators as-is, validate later
            _ => Operator::Unknown(self.to_string()),
        }
    }
}

/// Convenience module for operator constants
pub mod op {
    use super::Operator;
    
    pub const GT: Operator = Operator::GT;
    pub const LT: Operator = Operator::LT;
    pub const EQ: Operator = Operator::EQ;
    pub const NEQ: Operator = Operator::NEQ;
    pub const GTE: Operator = Operator::GTE;
    pub const LTE: Operator = Operator::LTE;
    pub const LIKE: Operator = Operator::LIKE;
    pub const ILIKE: Operator = Operator::ILIKE;
    pub const IN: Operator = Operator::IN;
    pub const NOT_IN: Operator = Operator::NOT_IN;
    pub const IS_NULL: Operator = Operator::IS_NULL;
    pub const IS_NOT_NULL: Operator = Operator::IS_NOT_NULL;
    pub const EXISTS: Operator = Operator::EXISTS;
    pub const NOT_EXISTS: Operator = Operator::NOT_EXISTS;
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_operator_constants() {
        assert_eq!(Operator::GT.as_str(), ">");
        assert_eq!(Operator::LT.as_str(), "<");
        assert_eq!(Operator::EQ.as_str(), "=");
        assert_eq!(Operator::LIKE.as_str(), "LIKE");
    }
    
    #[test]
    fn test_custom_operator() {
        let custom_op = Operator::custom("@@");
        assert_eq!(custom_op.as_str(), "@@");
    }
    
    #[test]
    fn test_display() {
        assert_eq!(format!("{}", Operator::GT), ">");
        assert_eq!(format!("{}", Operator::LIKE), "LIKE");
    }
    
    #[test]
    fn test_string_conversion() {
        assert_eq!(">".into_operator(), Operator::GT);
        assert_eq!("LIKE".into_operator(), Operator::LIKE);
        assert_eq!("like".into_operator(), Operator::LIKE);
        assert_eq!(">=".into_operator(), Operator::GTE);
    }
    
    #[test]
    fn test_invalid_string_conversion() {
        let invalid_op = "INVALID".into_operator();
        assert_eq!(invalid_op, Operator::Unknown("INVALID".to_string()));
        
        // Test that validation fails
        assert!(invalid_op.validate().is_err());
    }
    
    #[test]
    fn test_operator_equality() {
        assert_eq!(Operator::GT, ">".into_operator());
        assert_eq!(Operator::LIKE, "like".into_operator());
    }
    
    #[test]
    fn test_null_operators() {
        assert_eq!("IS NULL".into_operator(), Operator::IS_NULL);
        assert_eq!("is null".into_operator(), Operator::IS_NULL);
        assert_eq!("IS NOT NULL".into_operator(), Operator::IS_NOT_NULL);
    }
    
    #[test]
    fn test_deferred_validation_in_query() {
        use crate::{table, builder::QueryBuilder};
        
        // Creating a query with invalid operator should not panic
        let query = table("users").where_(("age", "INVALID_OP", 18));
        
        // But generating SQL should fail
        assert!(query.to_sql().is_err());
        
        let err = query.to_sql().unwrap_err();
        assert!(err.to_string().contains("Unknown operator 'INVALID_OP'"));
    }
}