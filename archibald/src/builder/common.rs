//! Common types and traits shared across all query builders

use crate::{IntoOperator, Operator, Result, Value};

/// Core trait for all query builders
pub trait QueryBuilder {
    /// Generate the SQL query string
    fn to_sql(&self) -> Result<String>;

    /// Get the parameters for the query
    fn parameters(&self) -> &[Value];

    /// Clone the builder (for immutable chaining)
    fn clone_builder(&self) -> Self
    where
        Self: Sized;
}

/// Trait for conditions that can be used in WHERE clauses
pub trait IntoCondition {
    fn into_condition(self) -> (String, Operator, Value);
}

// Implementation for shorthand equality: where(("age", 18))
impl<T> IntoCondition for (&str, T)
where
    T: Into<Value>,
{
    fn into_condition(self) -> (String, Operator, Value) {
        (self.0.to_string(), Operator::EQ, self.1.into())
    }
}

// Implementation for explicit operators: where(("age", op::GT, 18)) or where(("age", ">", 18))
impl<T, O> IntoCondition for (&str, O, T)
where
    T: Into<Value>,
    O: IntoOperator,
{
    fn into_condition(self) -> (String, Operator, Value) {
        (self.0.to_string(), self.1.into_operator(), self.2.into())
    }
}

/// A WHERE condition
#[derive(Debug, Clone, PartialEq)]
pub struct WhereCondition {
    pub column: String,
    pub operator: Operator,
    pub value: Value,
    pub connector: WhereConnector,
}

/// How WHERE conditions are connected
#[derive(Debug, Clone, PartialEq)]
pub enum WhereConnector {
    And,
    Or,
}

/// Aggregation function types
#[derive(Debug, Clone, PartialEq)]
pub enum AggregateFunction {
    Count,
    CountDistinct,
    Sum,
    Avg,
    Min,
    Max,
}

impl std::fmt::Display for AggregateFunction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AggregateFunction::Count => write!(f, "COUNT"),
            AggregateFunction::CountDistinct => write!(f, "COUNT(DISTINCT"),
            AggregateFunction::Sum => write!(f, "SUM"),
            AggregateFunction::Avg => write!(f, "AVG"),
            AggregateFunction::Min => write!(f, "MIN"),
            AggregateFunction::Max => write!(f, "MAX"),
        }
    }
}

/// Column selector that can be a regular column or an aggregation
#[derive(Debug, Clone)]
pub enum ColumnSelector {
    Column(String),
    Aggregate {
        function: AggregateFunction,
        column: String,
        alias: Option<String>,
    },
    CountAll {
        alias: Option<String>,
    },
    // SubqueryColumn will be handled in select.rs due to circular dependency
}

impl ColumnSelector {
    /// Create a COUNT(*) selector
    pub fn count() -> Self {
        Self::CountAll { alias: None }
    }

    /// Create a COUNT(*) selector with alias
    pub fn count_as(alias: &str) -> Self {
        Self::CountAll {
            alias: Some(alias.to_string()),
        }
    }

    /// Create a COUNT(column) selector
    pub fn count_column(column: &str) -> Self {
        Self::Aggregate {
            function: AggregateFunction::Count,
            column: column.to_string(),
            alias: None,
        }
    }

    /// Create a COUNT(DISTINCT column) selector
    pub fn count_distinct(column: &str) -> Self {
        Self::Aggregate {
            function: AggregateFunction::CountDistinct,
            column: column.to_string(),
            alias: None,
        }
    }

    /// Create a SUM(column) selector
    pub fn sum(column: &str) -> Self {
        Self::Aggregate {
            function: AggregateFunction::Sum,
            column: column.to_string(),
            alias: None,
        }
    }

    /// Create an AVG(column) selector
    pub fn avg(column: &str) -> Self {
        Self::Aggregate {
            function: AggregateFunction::Avg,
            column: column.to_string(),
            alias: None,
        }
    }

    /// Create a MIN(column) selector
    pub fn min(column: &str) -> Self {
        Self::Aggregate {
            function: AggregateFunction::Min,
            column: column.to_string(),
            alias: None,
        }
    }

    /// Create a MAX(column) selector
    pub fn max(column: &str) -> Self {
        Self::Aggregate {
            function: AggregateFunction::Max,
            column: column.to_string(),
            alias: None,
        }
    }

    /// Add alias to this column selector
    pub fn as_alias(mut self, alias: &str) -> Self {
        match self {
            Self::Column(_) => {
                // For regular columns, we can't add an alias directly to the enum variant
                // This would require restructuring the enum or handling it differently
                self
            }
            Self::Aggregate {
                alias: ref mut alias_field,
                ..
            } => {
                *alias_field = Some(alias.to_string());
                self
            }
            Self::CountAll {
                alias: ref mut alias_field,
            } => {
                *alias_field = Some(alias.to_string());
                self
            }
            // SubqueryColumn handled in select.rs
        }
    }

    // Subquery selectors will be handled in select.rs
}

/// Trait to convert various types into columns
pub trait IntoColumns {
    fn into_columns(self) -> Vec<String>;
}

impl IntoColumns for &str {
    fn into_columns(self) -> Vec<String> {
        vec![self.to_string()]
    }
}

impl IntoColumns for String {
    fn into_columns(self) -> Vec<String> {
        vec![self]
    }
}

impl IntoColumns for Vec<String> {
    fn into_columns(self) -> Vec<String> {
        self
    }
}

impl IntoColumns for Vec<&str> {
    fn into_columns(self) -> Vec<String> {
        self.into_iter().map(|s| s.to_string()).collect()
    }
}

// For tuples
impl IntoColumns for (&str, &str) {
    fn into_columns(self) -> Vec<String> {
        vec![self.0.to_string(), self.1.to_string()]
    }
}

impl IntoColumns for (&str, &str, &str) {
    fn into_columns(self) -> Vec<String> {
        vec![self.0.to_string(), self.1.to_string(), self.2.to_string()]
    }
}

impl IntoColumns for (&str, &str, &str, &str) {
    fn into_columns(self) -> Vec<String> {
        vec![
            self.0.to_string(),
            self.1.to_string(),
            self.2.to_string(),
            self.3.to_string(),
        ]
    }
}

impl IntoColumns for (&str, &str, &str, &str, &str) {
    fn into_columns(self) -> Vec<String> {
        vec![
            self.0.to_string(),
            self.1.to_string(),
            self.2.to_string(),
            self.3.to_string(),
            self.4.to_string(),
        ]
    }
}

/// JOIN types
#[derive(Debug, Clone, PartialEq)]
pub enum JoinType {
    Inner,
    Left,
    Right,
    Full,
    Cross,
}

impl std::fmt::Display for JoinType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            JoinType::Inner => write!(f, "INNER"),
            JoinType::Left => write!(f, "LEFT"),
            JoinType::Right => write!(f, "RIGHT"),
            JoinType::Full => write!(f, "FULL OUTER"),
            JoinType::Cross => write!(f, "CROSS"),
        }
    }
}

/// How JOIN conditions are connected
#[derive(Debug, Clone, PartialEq)]
pub enum JoinConnector {
    And,
    Or,
}

/// A condition in a JOIN ON clause
#[derive(Debug, Clone, PartialEq)]
pub struct JoinCondition {
    pub left_column: String,
    pub operator: Operator,
    pub right_column: String,
    pub connector: JoinConnector,
}

/// A complete JOIN clause with table and conditions
#[derive(Debug, Clone, PartialEq)]
pub struct JoinClause {
    pub join_type: JoinType,
    pub table: String,
    pub on_conditions: Vec<JoinCondition>,
}

/// Sort direction for ORDER BY clauses
#[derive(Debug, Clone, PartialEq)]
pub enum SortDirection {
    Asc,
    Desc,
}

impl std::fmt::Display for SortDirection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SortDirection::Asc => write!(f, "ASC"),
            SortDirection::Desc => write!(f, "DESC"),
        }
    }
}

/// An ORDER BY clause
#[derive(Debug, Clone, PartialEq)]
pub struct OrderByClause {
    pub column: String,
    pub direction: SortDirection,
}

/// A GROUP BY clause
#[derive(Debug, Clone, PartialEq)]
pub struct GroupByClause {
    pub columns: Vec<String>,
}

/// A HAVING condition (used with GROUP BY)
#[derive(Debug, Clone, PartialEq)]
pub struct HavingCondition {
    pub column_or_function: String,
    pub operator: Operator,
    pub value: Value,
    pub connector: WhereConnector,
}

/// Trait to convert various types into column selectors
pub trait IntoColumnSelectors {
    fn into_column_selectors(self) -> Vec<crate::ColumnSelector>;
}

impl IntoColumnSelectors for &str {
    fn into_column_selectors(self) -> Vec<crate::ColumnSelector> {
        vec![crate::ColumnSelector::Column(self.to_string())]
    }
}

impl IntoColumnSelectors for String {
    fn into_column_selectors(self) -> Vec<crate::ColumnSelector> {
        vec![crate::ColumnSelector::Column(self)]
    }
}

impl IntoColumnSelectors for Vec<String> {
    fn into_column_selectors(self) -> Vec<crate::ColumnSelector> {
        self.into_iter()
            .map(|s| crate::ColumnSelector::Column(s))
            .collect()
    }
}

impl IntoColumnSelectors for Vec<&str> {
    fn into_column_selectors(self) -> Vec<crate::ColumnSelector> {
        self.into_iter()
            .map(|s| crate::ColumnSelector::Column(s.to_string()))
            .collect()
    }
}

impl IntoColumnSelectors for crate::ColumnSelector {
    fn into_column_selectors(self) -> Vec<crate::ColumnSelector> {
        vec![self]
    }
}

impl IntoColumnSelectors for Vec<crate::ColumnSelector> {
    fn into_column_selectors(self) -> Vec<crate::ColumnSelector> {
        self
    }
}

// Tuple implementations for IntoColumnSelectors
impl IntoColumnSelectors for (&str, &str) {
    fn into_column_selectors(self) -> Vec<crate::ColumnSelector> {
        vec![
            crate::ColumnSelector::Column(self.0.to_string()),
            crate::ColumnSelector::Column(self.1.to_string()),
        ]
    }
}

impl IntoColumnSelectors for (&str, &str, &str) {
    fn into_column_selectors(self) -> Vec<crate::ColumnSelector> {
        vec![
            crate::ColumnSelector::Column(self.0.to_string()),
            crate::ColumnSelector::Column(self.1.to_string()),
            crate::ColumnSelector::Column(self.2.to_string()),
        ]
    }
}

impl IntoColumnSelectors for (&str, &str, &str, &str) {
    fn into_column_selectors(self) -> Vec<crate::ColumnSelector> {
        vec![
            crate::ColumnSelector::Column(self.0.to_string()),
            crate::ColumnSelector::Column(self.1.to_string()),
            crate::ColumnSelector::Column(self.2.to_string()),
            crate::ColumnSelector::Column(self.3.to_string()),
        ]
    }
}

impl IntoColumnSelectors for (&str, &str, &str, &str, &str) {
    fn into_column_selectors(self) -> Vec<crate::ColumnSelector> {
        vec![
            crate::ColumnSelector::Column(self.0.to_string()),
            crate::ColumnSelector::Column(self.1.to_string()),
            crate::ColumnSelector::Column(self.2.to_string()),
            crate::ColumnSelector::Column(self.3.to_string()),
            crate::ColumnSelector::Column(self.4.to_string()),
        ]
    }
}

// Support mixed tuples with ColumnSelectors
impl IntoColumnSelectors for (&str, crate::ColumnSelector) {
    fn into_column_selectors(self) -> Vec<crate::ColumnSelector> {
        vec![crate::ColumnSelector::Column(self.0.to_string()), self.1]
    }
}

impl IntoColumnSelectors for (&str, crate::ColumnSelector, crate::ColumnSelector) {
    fn into_column_selectors(self) -> Vec<crate::ColumnSelector> {
        vec![
            crate::ColumnSelector::Column(self.0.to_string()),
            self.1,
            self.2,
        ]
    }
}

impl IntoColumnSelectors for (crate::ColumnSelector, &str, crate::ColumnSelector) {
    fn into_column_selectors(self) -> Vec<crate::ColumnSelector> {
        vec![
            self.0,
            crate::ColumnSelector::Column(self.1.to_string()),
            self.2,
        ]
    }
}

// Forward declarations - these will be defined in select.rs
// pub struct Subquery;
// pub struct SubqueryCondition;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::operator::op;

    #[test]
    fn test_string_operator_conversion() {
        // Test that string operators work in conditions
        let condition = ("age", ">", 18);
        let (column, operator, value) = condition.into_condition();
        assert_eq!(column, "age");
        assert_eq!(operator, op::GT);
        assert_eq!(value, 18.into());
    }

    #[test]
    fn test_condition_trait_implementations() {
        // Test shorthand equality
        let condition = ("name", "John");
        let (column, operator, value) = condition.into_condition();
        assert_eq!(column, "name");
        assert_eq!(operator, op::EQ);
        assert_eq!(value, "John".into());

        // Test explicit operators
        let condition = ("age", op::GT, 18);
        let (column, operator, value) = condition.into_condition();
        assert_eq!(column, "age");
        assert_eq!(operator, op::GT);
        assert_eq!(value, 18.into());
    }

    #[test]
    fn test_into_columns_implementations() {
        // Single string
        let cols = "name".into_columns();
        assert_eq!(cols, vec!["name"]);

        // Tuple
        let cols = ("name", "age").into_columns();
        assert_eq!(cols, vec!["name", "age"]);

        // Vector
        let cols = vec!["name", "age"].into_columns();
        assert_eq!(cols, vec!["name", "age"]);
    }
}
