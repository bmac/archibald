//! SQL operator types and conversions

use std::fmt::{self, Display};

/// Type-safe SQL operator
#[derive(Debug, Clone, PartialEq, Eq)]
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
        Operator(op)
    }
    
    /// Get the string representation of the operator
    pub fn as_str(&self) -> &str {
        self.0
    }
}

impl Display for Operator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
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

/// Allow string literals for common SQL operators with validation
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
            _ => panic!(
                "Unknown operator '{}'. Use Operator::{} constants or Operator::custom(\"{}\") for custom operators.", 
                self, 
                self.to_uppercase().replace(" ", "_").replace("!", "N"), 
                self
            ),
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
    #[should_panic(expected = "Unknown operator 'INVALID'")]
    fn test_invalid_string_conversion() {
        "INVALID".into_operator();
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
}