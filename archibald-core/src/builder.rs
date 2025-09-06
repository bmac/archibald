//! Query builder traits and implementations

use crate::{Result, Operator, IntoOperator, Value};

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
    T: Into<Value>
{
    fn into_condition(self) -> (String, Operator, Value) {
        (self.0.to_string(), Operator::EQ, self.1.into())
    }
}

// Implementation for explicit operators: where(("age", op::GT, 18)) or where(("age", ">", 18))
impl<T, O> IntoCondition for (&str, O, T) 
where 
    T: Into<Value>,
    O: IntoOperator
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
#[derive(Debug, Clone, PartialEq)]
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
}

impl ColumnSelector {
    /// Create a COUNT(*) selector
    pub fn count() -> Self {
        Self::CountAll { alias: None }
    }
    
    /// Create a COUNT(*) selector with alias
    pub fn count_as(alias: &str) -> Self {
        Self::CountAll { 
            alias: Some(alias.to_string()) 
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
    
    /// Add an alias to any selector
    pub fn as_alias(mut self, alias: &str) -> Self {
        match &mut self {
            Self::Column(_) => {
                // Convert to aggregate with alias (this is a simplification)
                // In practice, we'd want a separate Column variant with alias
                self
            }
            Self::Aggregate { alias: a, .. } => {
                *a = Some(alias.to_string());
                self
            }
            Self::CountAll { alias: a } => {
                *a = Some(alias.to_string());
                self
            }
        }
    }
    
    /// Convert to SQL string
    pub fn to_sql(&self) -> String {
        match self {
            Self::Column(name) => name.clone(),
            Self::Aggregate { function, column, alias } => {
                let func_sql = match function {
                    AggregateFunction::CountDistinct => {
                        format!("COUNT(DISTINCT {})", column)
                    }
                    _ => {
                        format!("{}({})", function, column)
                    }
                };
                
                if let Some(alias) = alias {
                    format!("{} AS {}", func_sql, alias)
                } else {
                    func_sql
                }
            }
            Self::CountAll { alias } => {
                let sql = "COUNT(*)".to_string();
                if let Some(alias) = alias {
                    format!("{} AS {}", sql, alias)
                } else {
                    sql
                }
            }
        }
    }
}

/// JOIN clause types
#[derive(Debug, Clone, PartialEq)]
pub enum JoinType {
    Inner,
    Left,
    Right,
    FullOuter,
    Cross,
}

impl std::fmt::Display for JoinType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            JoinType::Inner => write!(f, "INNER JOIN"),
            JoinType::Left => write!(f, "LEFT JOIN"),
            JoinType::Right => write!(f, "RIGHT JOIN"),
            JoinType::FullOuter => write!(f, "FULL OUTER JOIN"),
            JoinType::Cross => write!(f, "CROSS JOIN"),
        }
    }
}

/// A JOIN clause
#[derive(Debug, Clone, PartialEq)]
pub struct JoinClause {
    pub join_type: JoinType,
    pub table: String,
    pub on_conditions: Vec<JoinCondition>,
}

/// How JOIN conditions are connected
#[derive(Debug, Clone, PartialEq)]
pub enum JoinConnector {
    And,
    Or,
}

/// JOIN ON condition
#[derive(Debug, Clone, PartialEq)]
pub struct JoinCondition {
    pub left_column: String,
    pub operator: Operator,
    pub right_column: String,
    pub connector: JoinConnector,
}

/// Sort direction for ORDER BY clause
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

/// ORDER BY clause
#[derive(Debug, Clone, PartialEq)]
pub struct OrderByClause {
    pub column: String,
    pub direction: SortDirection,
}

/// GROUP BY clause
#[derive(Debug, Clone, PartialEq)]
pub struct GroupByClause {
    pub columns: Vec<String>,
}

/// HAVING condition (similar to WHERE but for aggregated results)
#[derive(Debug, Clone, PartialEq)]
pub struct HavingCondition {
    pub column_or_function: String,
    pub operator: Operator,
    pub value: Value,
    pub connector: WhereConnector,
}

/// SELECT query builder
#[derive(Debug, Clone)]
pub struct SelectBuilder {
    table_name: String,
    selected_columns: Vec<ColumnSelector>,
    where_conditions: Vec<WhereCondition>,
    join_clauses: Vec<JoinClause>,
    order_by_clauses: Vec<OrderByClause>,
    group_by_clause: Option<GroupByClause>,
    having_conditions: Vec<HavingCondition>,
    distinct: bool,
    limit_value: Option<u64>,
    offset_value: Option<u64>,
    parameters: Vec<Value>,
}

impl SelectBuilder {
    /// Create a new SELECT query builder
    pub fn new(table: &str) -> Self {
        Self {
            table_name: table.to_string(),
            selected_columns: vec![ColumnSelector::Column("*".to_string())],
            where_conditions: Vec::new(),
            join_clauses: Vec::new(),
            order_by_clauses: Vec::new(),
            group_by_clause: None,
            having_conditions: Vec::new(),
            distinct: false,
            limit_value: None,
            offset_value: None,
            parameters: Vec::new(),
        }
    }
    
    /// Select specific columns
    /// 
    /// # Examples
    /// ```
    /// use archibald_core::table;
    /// 
    /// let query = table("users").select(("id", "name", "email"));
    /// ```
    pub fn select<T>(mut self, columns: T) -> Self
    where
        T: IntoColumnSelectors,
    {
        self.selected_columns = columns.into_column_selectors();
        self
    }
    
    /// Select all columns (equivalent to SELECT *)
    pub fn select_all(mut self) -> Self {
        self.selected_columns = vec![ColumnSelector::Column("*".to_string())];
        self
    }
    
    /// Add a WHERE condition
    /// 
    /// # Examples
    /// ```
    /// use archibald_core::{table, op};
    /// 
    /// let query = table("users")
    ///     .where_(("age", op::GT, 18))
    ///     .where_(("name", "John"));
    /// ```
    pub fn where_<C>(mut self, condition: C) -> Self
    where
        C: IntoCondition,
    {
        let (column, operator, value) = condition.into_condition();
        
        self.where_conditions.push(WhereCondition {
            column,
            operator,
            value,
            connector: WhereConnector::And,
        });
        
        self
    }
    
    /// Add an OR WHERE condition
    pub fn or_where<C>(mut self, condition: C) -> Self
    where
        C: IntoCondition,
    {
        let (column, operator, value) = condition.into_condition();
        
        self.where_conditions.push(WhereCondition {
            column,
            operator,
            value,
            connector: WhereConnector::Or,
        });
        
        self
    }
    
    /// Add an AND WHERE condition (same as where)
    pub fn and_where<C>(self, condition: C) -> Self
    where
        C: IntoCondition,
    {
        self.where_(condition)
    }
    
    /// Set the LIMIT clause
    pub fn limit(mut self, limit: u64) -> Self {
        self.limit_value = Some(limit);
        self
    }
    
    /// Set the OFFSET clause
    pub fn offset(mut self, offset: u64) -> Self {
        self.offset_value = Some(offset);
        self
    }
    
    /// Add an INNER JOIN clause
    /// 
    /// # Examples
    /// ```
    /// use archibald_core::table;
    /// 
    /// let query = table("users")
    ///     .inner_join("posts", "users.id", "posts.user_id");
    /// ```
    pub fn inner_join(mut self, table: &str, left_col: &str, right_col: &str) -> Self {
        self.join_clauses.push(JoinClause {
            join_type: JoinType::Inner,
            table: table.to_string(),
            on_conditions: vec![JoinCondition {
                left_column: left_col.to_string(),
                operator: Operator::EQ,
                right_column: right_col.to_string(),
                connector: JoinConnector::And,
            }],
        });
        self
    }
    
    /// Add a LEFT JOIN clause
    pub fn left_join(mut self, table: &str, left_col: &str, right_col: &str) -> Self {
        self.join_clauses.push(JoinClause {
            join_type: JoinType::Left,
            table: table.to_string(),
            on_conditions: vec![JoinCondition {
                left_column: left_col.to_string(),
                operator: Operator::EQ,
                right_column: right_col.to_string(),
                connector: JoinConnector::And,
            }],
        });
        self
    }
    
    /// Add a RIGHT JOIN clause
    pub fn right_join(mut self, table: &str, left_col: &str, right_col: &str) -> Self {
        self.join_clauses.push(JoinClause {
            join_type: JoinType::Right,
            table: table.to_string(),
            on_conditions: vec![JoinCondition {
                left_column: left_col.to_string(),
                operator: Operator::EQ,
                right_column: right_col.to_string(),
                connector: JoinConnector::And,
            }],
        });
        self
    }
    
    /// Add a FULL OUTER JOIN clause
    pub fn full_outer_join(mut self, table: &str, left_col: &str, right_col: &str) -> Self {
        self.join_clauses.push(JoinClause {
            join_type: JoinType::FullOuter,
            table: table.to_string(),
            on_conditions: vec![JoinCondition {
                left_column: left_col.to_string(),
                operator: Operator::EQ,
                right_column: right_col.to_string(),
                connector: JoinConnector::And,
            }],
        });
        self
    }
    
    /// Add a CROSS JOIN clause
    pub fn cross_join(mut self, table: &str) -> Self {
        self.join_clauses.push(JoinClause {
            join_type: JoinType::Cross,
            table: table.to_string(),
            on_conditions: Vec::new(), // CROSS JOIN has no ON conditions
        });
        self
    }
    
    /// Generic JOIN method with custom join type and operator
    /// 
    /// # Examples
    /// ```
    /// use archibald_core::{table, JoinType, op};
    /// 
    /// let query = table("users")
    ///     .join(JoinType::Left, "profiles", "users.id", op::EQ, "profiles.user_id");
    /// ```
    pub fn join<O>(mut self, join_type: JoinType, table: &str, left_col: &str, operator: O, right_col: &str) -> Self
    where
        O: IntoOperator,
    {
        self.join_clauses.push(JoinClause {
            join_type,
            table: table.to_string(),
            on_conditions: vec![JoinCondition {
                left_column: left_col.to_string(),
                operator: operator.into_operator(),
                right_column: right_col.to_string(),
                connector: JoinConnector::And,
            }],
        });
        self
    }
    
    /// Add ORDER BY clause with ascending sort
    /// 
    /// # Examples
    /// ```
    /// use archibald_core::table;
    /// 
    /// let query = table("users").order_by("name");
    /// ```
    pub fn order_by(mut self, column: &str) -> Self {
        self.order_by_clauses.push(OrderByClause {
            column: column.to_string(),
            direction: SortDirection::Asc,
        });
        self
    }
    
    /// Add ORDER BY clause with descending sort
    /// 
    /// # Examples
    /// ```
    /// use archibald_core::table;
    /// 
    /// let query = table("users").order_by_desc("created_at");
    /// ```
    pub fn order_by_desc(mut self, column: &str) -> Self {
        self.order_by_clauses.push(OrderByClause {
            column: column.to_string(),
            direction: SortDirection::Desc,
        });
        self
    }
    
    /// Add ORDER BY clause with custom direction
    /// 
    /// # Examples
    /// ```
    /// use archibald_core::{table, SortDirection};
    /// 
    /// let query = table("users").order_by_with_direction("name", SortDirection::Desc);
    /// ```
    pub fn order_by_with_direction(mut self, column: &str, direction: SortDirection) -> Self {
        self.order_by_clauses.push(OrderByClause {
            column: column.to_string(),
            direction,
        });
        self
    }
    
    /// Add GROUP BY clause
    /// 
    /// # Examples
    /// ```
    /// use archibald_core::table;
    /// 
    /// let query = table("orders").group_by(("customer_id", "status"));
    /// ```
    pub fn group_by<C>(mut self, columns: C) -> Self 
    where 
        C: IntoColumns,
    {
        self.group_by_clause = Some(GroupByClause {
            columns: columns.into_columns(),
        });
        self
    }
    
    /// Add DISTINCT clause to eliminate duplicate rows
    /// 
    /// # Examples
    /// ```
    /// use archibald_core::table;
    /// 
    /// let query = table("users").select("status").distinct();
    /// ```
    pub fn distinct(mut self) -> Self {
        self.distinct = true;
        self
    }
    
    /// Add a HAVING condition for aggregated results
    /// 
    /// # Examples
    /// ```
    /// use archibald_core::{table, ColumnSelector, op};
    /// 
    /// let query = table("orders")
    ///     .select(vec![
    ///         ColumnSelector::Column("status".to_string()),
    ///         ColumnSelector::count().as_alias("count")
    ///     ])
    ///     .group_by("status")
    ///     .having(("COUNT(*)", op::GT, 5));
    /// ```
    pub fn having<C>(mut self, condition: C) -> Self
    where
        C: IntoCondition,
    {
        let (column, operator, value) = condition.into_condition();
        self.having_conditions.push(HavingCondition {
            column_or_function: column,
            operator,
            value,
            connector: WhereConnector::And,
        });
        self.parameters.push(self.having_conditions.last().unwrap().value.clone());
        self
    }
    
    /// Add an AND HAVING condition
    pub fn and_having<C>(mut self, condition: C) -> Self
    where
        C: IntoCondition,
    {
        let (column, operator, value) = condition.into_condition();
        self.having_conditions.push(HavingCondition {
            column_or_function: column,
            operator,
            value,
            connector: WhereConnector::And,
        });
        self.parameters.push(self.having_conditions.last().unwrap().value.clone());
        self
    }
    
    /// Add an OR HAVING condition
    pub fn or_having<C>(mut self, condition: C) -> Self
    where
        C: IntoCondition,
    {
        let (column, operator, value) = condition.into_condition();
        self.having_conditions.push(HavingCondition {
            column_or_function: column,
            operator,
            value,
            connector: WhereConnector::Or,
        });
        self.parameters.push(self.having_conditions.last().unwrap().value.clone());
        self
    }
}

impl QueryBuilder for SelectBuilder {
    fn to_sql(&self) -> Result<String> {
        let mut sql = String::new();
        
        // SELECT clause
        sql.push_str("SELECT ");
        if self.distinct {
            sql.push_str("DISTINCT ");
        }
        let column_strs: Vec<String> = self.selected_columns.iter().map(|col| col.to_sql()).collect();
        sql.push_str(&column_strs.join(", "));
        
        // FROM clause
        sql.push_str(" FROM ");
        sql.push_str(&self.table_name);
        
        // JOIN clauses
        for join_clause in &self.join_clauses {
            sql.push(' ');
            sql.push_str(match join_clause.join_type {
                JoinType::Inner => "INNER JOIN",
                JoinType::Left => "LEFT JOIN",
                JoinType::Right => "RIGHT JOIN",
                JoinType::FullOuter => "FULL OUTER JOIN",
                JoinType::Cross => "CROSS JOIN",
            });
            sql.push(' ');
            sql.push_str(&join_clause.table);
            
            // Add ON conditions for non-CROSS joins
            if !matches!(join_clause.join_type, JoinType::Cross) && !join_clause.on_conditions.is_empty() {
                sql.push_str(" ON ");
                
                for (i, condition) in join_clause.on_conditions.iter().enumerate() {
                    if i > 0 {
                        match condition.connector {
                            JoinConnector::And => sql.push_str(" AND "),
                            JoinConnector::Or => sql.push_str(" OR "),
                        }
                    }
                    
                    sql.push_str(&condition.left_column);
                    sql.push(' ');
                    sql.push_str(condition.operator.as_str());
                    sql.push(' ');
                    sql.push_str(&condition.right_column);
                }
            }
        }
        
        // WHERE clause
        if !self.where_conditions.is_empty() {
            sql.push_str(" WHERE ");
            
            for (i, condition) in self.where_conditions.iter().enumerate() {
                if i > 0 {
                    match condition.connector {
                        WhereConnector::And => sql.push_str(" AND "),
                        WhereConnector::Or => sql.push_str(" OR "),
                    }
                }
                
                sql.push_str(&condition.column);
                sql.push(' ');
                sql.push_str(condition.operator.as_str());
                
                // For array values, use IN syntax
                if let Value::Array(_) = condition.value {
                    sql.push_str(" (");
                    // TODO: Handle parameter placeholders properly
                    sql.push_str("?");
                    sql.push(')');
                } else {
                    sql.push_str(" ?");
                }
            }
        }
        
        // GROUP BY clause
        if let Some(group_by) = &self.group_by_clause {
            sql.push_str(" GROUP BY ");
            sql.push_str(&group_by.columns.join(", "));
        }
        
        // HAVING clause
        if !self.having_conditions.is_empty() {
            sql.push_str(" HAVING ");
            
            for (i, condition) in self.having_conditions.iter().enumerate() {
                if i > 0 {
                    match condition.connector {
                        WhereConnector::And => sql.push_str(" AND "),
                        WhereConnector::Or => sql.push_str(" OR "),
                    }
                }
                
                sql.push_str(&condition.column_or_function);
                sql.push(' ');
                sql.push_str(condition.operator.as_str());
                
                // For array values, use IN syntax
                if let Value::Array(_) = condition.value {
                    sql.push_str(" (");
                    // TODO: Handle parameter placeholders properly
                    sql.push_str("?");
                    sql.push(')');
                } else {
                    sql.push_str(" ?");
                }
            }
        }
        
        // ORDER BY clause
        if !self.order_by_clauses.is_empty() {
            sql.push_str(" ORDER BY ");
            
            for (i, order_clause) in self.order_by_clauses.iter().enumerate() {
                if i > 0 {
                    sql.push_str(", ");
                }
                sql.push_str(&order_clause.column);
                sql.push(' ');
                sql.push_str(&order_clause.direction.to_string());
            }
        }
        
        // LIMIT clause
        if let Some(limit) = self.limit_value {
            sql.push_str(&format!(" LIMIT {}", limit));
        }
        
        // OFFSET clause
        if let Some(offset) = self.offset_value {
            sql.push_str(&format!(" OFFSET {}", offset));
        }
        
        Ok(sql)
    }
    
    fn parameters(&self) -> &[Value] {
        &self.parameters
    }
    
    fn clone_builder(&self) -> Self {
        self.clone()
    }
}

/// Trait for types that can be converted to column lists
pub trait IntoColumns {
    fn into_columns(self) -> Vec<String>;
}

/// Trait for types that can be converted to column selectors
pub trait IntoColumnSelectors {
    fn into_column_selectors(self) -> Vec<ColumnSelector>;
}

impl IntoColumns for &str {
    fn into_columns(self) -> Vec<String> {
        vec![self.to_string()]
    }
}

// IntoColumnSelectors implementations
impl IntoColumnSelectors for &str {
    fn into_column_selectors(self) -> Vec<ColumnSelector> {
        vec![ColumnSelector::Column(self.to_string())]
    }
}

impl IntoColumnSelectors for ColumnSelector {
    fn into_column_selectors(self) -> Vec<ColumnSelector> {
        vec![self]
    }
}

impl IntoColumnSelectors for Vec<ColumnSelector> {
    fn into_column_selectors(self) -> Vec<ColumnSelector> {
        self
    }
}

// Tuple implementations for IntoColumnSelectors  
impl IntoColumnSelectors for (&str,) {
    fn into_column_selectors(self) -> Vec<ColumnSelector> {
        vec![ColumnSelector::Column(self.0.to_string())]
    }
}

impl IntoColumnSelectors for (&str, &str) {
    fn into_column_selectors(self) -> Vec<ColumnSelector> {
        vec![
            ColumnSelector::Column(self.0.to_string()),
            ColumnSelector::Column(self.1.to_string())
        ]
    }
}

impl IntoColumnSelectors for (&str, &str, &str) {
    fn into_column_selectors(self) -> Vec<ColumnSelector> {
        vec![
            ColumnSelector::Column(self.0.to_string()),
            ColumnSelector::Column(self.1.to_string()),
            ColumnSelector::Column(self.2.to_string())
        ]
    }
}

impl IntoColumnSelectors for (&str, &str, &str, &str) {
    fn into_column_selectors(self) -> Vec<ColumnSelector> {
        vec![
            ColumnSelector::Column(self.0.to_string()),
            ColumnSelector::Column(self.1.to_string()),
            ColumnSelector::Column(self.2.to_string()),
            ColumnSelector::Column(self.3.to_string())
        ]
    }
}

impl IntoColumns for String {
    fn into_columns(self) -> Vec<String> {
        vec![self]
    }
}

impl IntoColumns for Vec<&str> {
    fn into_columns(self) -> Vec<String> {
        self.into_iter().map(|s| s.to_string()).collect()
    }
}

impl IntoColumns for Vec<String> {
    fn into_columns(self) -> Vec<String> {
        self
    }
}

// Implement for tuples of up to 8 columns (common use case)
impl IntoColumns for (&str,) {
    fn into_columns(self) -> Vec<String> {
        vec![self.0.to_string()]
    }
}

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
            self.3.to_string()
        ]
    }
}

/// INSERT query builder
#[derive(Debug, Clone)]
pub struct InsertBuilder {
    table_name: String,
    columns: Vec<String>,
    values: Vec<Vec<Value>>,
    parameters: Vec<Value>,
}

impl InsertBuilder {
    /// Create a new INSERT query builder
    pub fn new(table: &str) -> Self {
        Self {
            table_name: table.to_string(),
            columns: Vec::new(),
            values: Vec::new(),
            parameters: Vec::new(),
        }
    }
    
    /// Insert a single record
    /// 
    /// # Examples
    /// ```
    /// use archibald_core::InsertBuilder;
    /// use std::collections::HashMap;
    /// 
    /// let mut data = HashMap::new();
    /// data.insert("name".to_string(), "John".into());
    /// data.insert("age".to_string(), 30.into());
    /// 
    /// let query = InsertBuilder::new("users").insert(data);
    /// ```
    pub fn insert<T>(mut self, data: T) -> Self
    where
        T: IntoInsertData,
    {
        let (columns, values) = data.into_insert_data();
        self.columns = columns;
        self.values.push(values);
        self
    }
    
    /// Insert multiple records
    pub fn insert_many<T>(mut self, data: Vec<T>) -> Self
    where
        T: IntoInsertData + Clone,
    {
        if let Some(first) = data.first() {
            let (columns, _) = first.clone().into_insert_data();
            self.columns = columns;
            
            for item in data {
                let (_, values) = item.into_insert_data();
                self.values.push(values);
            }
        }
        self
    }
}

impl QueryBuilder for InsertBuilder {
    fn to_sql(&self) -> Result<String> {
        if self.columns.is_empty() || self.values.is_empty() {
            return Err(crate::Error::invalid_query("INSERT requires columns and values"));
        }
        
        let mut sql = String::new();
        
        // INSERT INTO clause
        sql.push_str("INSERT INTO ");
        sql.push_str(&self.table_name);
        
        // Columns
        sql.push_str(" (");
        sql.push_str(&self.columns.join(", "));
        sql.push_str(")");
        
        // VALUES clause
        sql.push_str(" VALUES ");
        let value_groups: Vec<String> = self.values
            .iter()
            .map(|row| {
                let placeholders: Vec<String> = row.iter().map(|_| "?".to_string()).collect();
                format!("({})", placeholders.join(", "))
            })
            .collect();
        sql.push_str(&value_groups.join(", "));
        
        Ok(sql)
    }
    
    fn parameters(&self) -> &[Value] {
        &self.parameters
    }
    
    fn clone_builder(&self) -> Self {
        self.clone()
    }
}

/// UPDATE query builder
#[derive(Debug, Clone)]
pub struct UpdateBuilder {
    table_name: String,
    set_clauses: Vec<(String, Value)>,
    where_conditions: Vec<WhereCondition>,
    parameters: Vec<Value>,
}

impl UpdateBuilder {
    /// Create a new UPDATE query builder
    pub fn new(table: &str) -> Self {
        Self {
            table_name: table.to_string(),
            set_clauses: Vec::new(),
            where_conditions: Vec::new(),
            parameters: Vec::new(),
        }
    }
    
    /// Set column values
    /// 
    /// # Examples
    /// ```
    /// use archibald_core::UpdateBuilder;
    /// use std::collections::HashMap;
    /// 
    /// let mut updates = HashMap::new();
    /// updates.insert("name".to_string(), "Jane".into());
    /// updates.insert("age".to_string(), 25.into());
    /// 
    /// let query = UpdateBuilder::new("users").set(updates);
    /// ```
    pub fn set<T>(mut self, data: T) -> Self
    where
        T: IntoUpdateData,
    {
        let updates = data.into_update_data();
        self.set_clauses.extend(updates);
        self
    }
    
    /// Add a WHERE condition
    pub fn where_<C>(mut self, condition: C) -> Self
    where
        C: IntoCondition,
    {
        let (column, operator, value) = condition.into_condition();
        
        self.where_conditions.push(WhereCondition {
            column,
            operator,
            value,
            connector: WhereConnector::And,
        });
        
        self
    }
    
    /// Add an OR WHERE condition
    pub fn or_where<C>(mut self, condition: C) -> Self
    where
        C: IntoCondition,
    {
        let (column, operator, value) = condition.into_condition();
        
        self.where_conditions.push(WhereCondition {
            column,
            operator,
            value,
            connector: WhereConnector::Or,
        });
        
        self
    }
    
    /// Add an AND WHERE condition (same as where_)
    pub fn and_where<C>(self, condition: C) -> Self
    where
        C: IntoCondition,
    {
        self.where_(condition)
    }
}

impl QueryBuilder for UpdateBuilder {
    fn to_sql(&self) -> Result<String> {
        if self.set_clauses.is_empty() {
            return Err(crate::Error::invalid_query("UPDATE requires SET clauses"));
        }
        
        let mut sql = String::new();
        
        // UPDATE clause
        sql.push_str("UPDATE ");
        sql.push_str(&self.table_name);
        
        // SET clause
        sql.push_str(" SET ");
        let set_parts: Vec<String> = self.set_clauses
            .iter()
            .map(|(column, _)| format!("{} = ?", column))
            .collect();
        sql.push_str(&set_parts.join(", "));
        
        // WHERE clause
        if !self.where_conditions.is_empty() {
            sql.push_str(" WHERE ");
            
            for (i, condition) in self.where_conditions.iter().enumerate() {
                if i > 0 {
                    match condition.connector {
                        WhereConnector::And => sql.push_str(" AND "),
                        WhereConnector::Or => sql.push_str(" OR "),
                    }
                }
                
                sql.push_str(&condition.column);
                sql.push(' ');
                sql.push_str(condition.operator.as_str());
                sql.push_str(" ?");
            }
        }
        
        Ok(sql)
    }
    
    fn parameters(&self) -> &[Value] {
        &self.parameters
    }
    
    fn clone_builder(&self) -> Self {
        self.clone()
    }
}

/// DELETE query builder
#[derive(Debug, Clone)]
pub struct DeleteBuilder {
    table_name: String,
    where_conditions: Vec<WhereCondition>,
    parameters: Vec<Value>,
}

impl DeleteBuilder {
    /// Create a new DELETE query builder
    pub fn new(table: &str) -> Self {
        Self {
            table_name: table.to_string(),
            where_conditions: Vec::new(),
            parameters: Vec::new(),
        }
    }
    
    /// Add a WHERE condition
    pub fn where_<C>(mut self, condition: C) -> Self
    where
        C: IntoCondition,
    {
        let (column, operator, value) = condition.into_condition();
        
        self.where_conditions.push(WhereCondition {
            column,
            operator,
            value,
            connector: WhereConnector::And,
        });
        
        self
    }
    
    /// Add an OR WHERE condition
    pub fn or_where<C>(mut self, condition: C) -> Self
    where
        C: IntoCondition,
    {
        let (column, operator, value) = condition.into_condition();
        
        self.where_conditions.push(WhereCondition {
            column,
            operator,
            value,
            connector: WhereConnector::Or,
        });
        
        self
    }
    
    /// Add an AND WHERE condition (same as where_)
    pub fn and_where<C>(self, condition: C) -> Self
    where
        C: IntoCondition,
    {
        self.where_(condition)
    }
}

impl QueryBuilder for DeleteBuilder {
    fn to_sql(&self) -> Result<String> {
        let mut sql = String::new();
        
        // DELETE FROM clause
        sql.push_str("DELETE FROM ");
        sql.push_str(&self.table_name);
        
        // WHERE clause
        if !self.where_conditions.is_empty() {
            sql.push_str(" WHERE ");
            
            for (i, condition) in self.where_conditions.iter().enumerate() {
                if i > 0 {
                    match condition.connector {
                        WhereConnector::And => sql.push_str(" AND "),
                        WhereConnector::Or => sql.push_str(" OR "),
                    }
                }
                
                sql.push_str(&condition.column);
                sql.push(' ');
                sql.push_str(condition.operator.as_str());
                sql.push_str(" ?");
            }
        }
        
        Ok(sql)
    }
    
    fn parameters(&self) -> &[Value] {
        &self.parameters
    }
    
    fn clone_builder(&self) -> Self {
        self.clone()
    }
}

/// Trait for types that can be converted to INSERT data
pub trait IntoInsertData {
    fn into_insert_data(self) -> (Vec<String>, Vec<Value>);
}

impl IntoInsertData for std::collections::HashMap<String, Value> {
    fn into_insert_data(self) -> (Vec<String>, Vec<Value>) {
        let columns: Vec<String> = self.keys().cloned().collect();
        let values: Vec<Value> = columns.iter().map(|k| self[k].clone()).collect();
        (columns, values)
    }
}

/// Trait for types that can be converted to UPDATE data
pub trait IntoUpdateData {
    fn into_update_data(self) -> Vec<(String, Value)>;
}

impl IntoUpdateData for std::collections::HashMap<String, Value> {
    fn into_update_data(self) -> Vec<(String, Value)> {
        self.into_iter().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::operator::op;
    
    #[test]
    fn test_basic_select() {
        let query = SelectBuilder::new("users");
        let sql = query.to_sql().unwrap();
        assert_eq!(sql, "SELECT * FROM users");
    }
    
    #[test]
    fn test_select_columns() {
        let query = SelectBuilder::new("users").select(("id", "name"));
        let sql = query.to_sql().unwrap();
        assert_eq!(sql, "SELECT id, name FROM users");
    }
    
    #[test]
    fn test_select_with_where() {
        let query = SelectBuilder::new("users").where_(("age", op::GT, 18));
        let sql = query.to_sql().unwrap();
        assert_eq!(sql, "SELECT * FROM users WHERE age > ?");
    }
    
    #[test]
    fn test_multiple_where_conditions() {
        let query = SelectBuilder::new("users")
            .where_(("age", op::GT, 18))
            .where_(("name", "John"));
        let sql = query.to_sql().unwrap();
        assert_eq!(sql, "SELECT * FROM users WHERE age > ? AND name = ?");
    }
    
    #[test]
    fn test_or_where() {
        let query = SelectBuilder::new("users")
            .where_(("age", op::GT, 18))
            .or_where(("status", "admin"));
        let sql = query.to_sql().unwrap();
        assert_eq!(sql, "SELECT * FROM users WHERE age > ? OR status = ?");
    }
    
    #[test]
    fn test_limit_and_offset() {
        let query = SelectBuilder::new("users")
            .limit(10)
            .offset(20);
        let sql = query.to_sql().unwrap();
        assert_eq!(sql, "SELECT * FROM users LIMIT 10 OFFSET 20");
    }
    
    #[test]
    fn test_string_operator_conversion() {
        let query = SelectBuilder::new("users").where_(("age", ">", 18));
        let sql = query.to_sql().unwrap();
        assert_eq!(sql, "SELECT * FROM users WHERE age > ?");
    }
    
    #[test]
    fn test_condition_trait_implementations() {
        // Test shorthand equality
        let (col, op, val) = ("age", 18).into_condition();
        assert_eq!(col, "age");
        assert_eq!(op, Operator::EQ);
        assert_eq!(val, Value::I32(18));
        
        // Test explicit operator
        let (col, op, val) = ("age", op::GT, 18).into_condition();
        assert_eq!(col, "age");
        assert_eq!(op, Operator::GT);
        assert_eq!(val, Value::I32(18));
        
        // Test string operator
        let (col, op, val) = ("name", "LIKE", "%john%").into_condition();
        assert_eq!(col, "name");
        assert_eq!(op, Operator::LIKE);
        assert_eq!(val, Value::String("%john%".to_string()));
    }
    
    #[test]
    fn test_immutable_builder_pattern() {
        let base_query = SelectBuilder::new("users");
        let query1 = base_query.clone().where_(("age", op::GT, 18));
        let query2 = base_query.clone().where_(("name", "John"));
        
        assert_ne!(query1.to_sql().unwrap(), query2.to_sql().unwrap());
    }
    
    #[test]
    fn test_insert_builder() {
        use std::collections::HashMap;
        
        let mut data = HashMap::new();
        data.insert("name".to_string(), Value::String("John".to_string()));
        data.insert("age".to_string(), Value::I32(30));
        
        let query = InsertBuilder::new("users").insert(data);
        let sql = query.to_sql().unwrap();
        
        // Note: HashMap iteration order is not guaranteed, so we just check structure
        assert!(sql.starts_with("INSERT INTO users ("));
        assert!(sql.contains(") VALUES ("));
        assert!(sql.contains("?, ?"));
    }
    
    #[test]
    fn test_insert_many() {
        use std::collections::HashMap;
        
        let mut data1 = HashMap::new();
        data1.insert("name".to_string(), Value::String("John".to_string()));
        data1.insert("age".to_string(), Value::I32(30));
        
        let mut data2 = HashMap::new();
        data2.insert("name".to_string(), Value::String("Jane".to_string()));
        data2.insert("age".to_string(), Value::I32(25));
        
        let query = InsertBuilder::new("users").insert_many(vec![data1, data2]);
        let sql = query.to_sql().unwrap();
        
        assert!(sql.starts_with("INSERT INTO users ("));
        assert!(sql.contains(") VALUES ("));
        assert!(sql.contains("), ("));
    }
    
    #[test]
    fn test_update_builder() {
        use std::collections::HashMap;
        
        let mut updates = HashMap::new();
        updates.insert("name".to_string(), Value::String("Jane".to_string()));
        updates.insert("age".to_string(), Value::I32(25));
        
        let query = UpdateBuilder::new("users")
            .set(updates)
            .where_(("id", op::EQ, 1));
        let sql = query.to_sql().unwrap();
        
        assert!(sql.starts_with("UPDATE users SET "));
        assert!(sql.contains(" WHERE id = ?"));
    }
    
    #[test]
    fn test_update_without_set_fails() {
        let query = UpdateBuilder::new("users").where_(("id", op::EQ, 1));
        let result = query.to_sql();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("UPDATE requires SET clauses"));
    }
    
    #[test]
    fn test_delete_builder() {
        let query = DeleteBuilder::new("users")
            .where_(("age", op::LT, 18))
            .or_where(("status", "inactive"));
        let sql = query.to_sql().unwrap();
        
        assert_eq!(sql, "DELETE FROM users WHERE age < ? OR status = ?");
    }
    
    #[test]
    fn test_delete_without_where() {
        let query = DeleteBuilder::new("users");
        let sql = query.to_sql().unwrap();
        assert_eq!(sql, "DELETE FROM users");
    }
    
    #[test]
    fn test_insert_empty_data_fails() {
        let query = InsertBuilder::new("users");
        let result = query.to_sql();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("INSERT requires columns and values"));
    }
    
    #[test]
    fn test_and_where_methods() {
        // Test that and_where works the same as where_
        let query1 = SelectBuilder::new("users")
            .where_(("age", op::GT, 18))
            .where_(("status", "active"));
            
        let query2 = SelectBuilder::new("users")
            .where_(("age", op::GT, 18))
            .and_where(("status", "active"));
            
        assert_eq!(query1.to_sql().unwrap(), query2.to_sql().unwrap());
        
        // Test with UpdateBuilder
        use std::collections::HashMap;
        let mut updates = HashMap::new();
        updates.insert("name".to_string(), Value::String("Test".to_string()));
        
        let update_query = UpdateBuilder::new("users")
            .set(updates)
            .where_(("id", 1))
            .and_where(("active", true));
        let sql = update_query.to_sql().unwrap();
        assert!(sql.contains("WHERE id = ? AND active = ?"));
        
        // Test with DeleteBuilder  
        let delete_query = DeleteBuilder::new("users")
            .where_(("age", op::LT, 18))
            .and_where(("status", "inactive"));
        let sql = delete_query.to_sql().unwrap();
        assert_eq!(sql, "DELETE FROM users WHERE age < ? AND status = ?");
    }
    
    #[test]
    fn test_complex_where_combinations() {
        let query = SelectBuilder::new("users")
            .where_(("age", op::GTE, 18))     // First condition (AND by default)
            .and_where(("status", "active"))  // Explicit AND
            .or_where(("role", "admin"))      // OR condition
            .and_where(("verified", true));   // Back to AND
            
        let sql = query.to_sql().unwrap();
        assert_eq!(sql, "SELECT * FROM users WHERE age >= ? AND status = ? OR role = ? AND verified = ?");
    }
    
    // JOIN operation tests
    #[test]
    fn test_inner_join() {
        let query = SelectBuilder::new("users")
            .select(("users.name", "profiles.bio"))
            .inner_join("profiles", "users.id", "profiles.user_id");
            
        let sql = query.to_sql().unwrap();
        assert_eq!(sql, "SELECT users.name, profiles.bio FROM users INNER JOIN profiles ON users.id = profiles.user_id");
    }
    
    #[test]
    fn test_left_join() {
        let query = SelectBuilder::new("users")
            .left_join("profiles", "users.id", "profiles.user_id");
            
        let sql = query.to_sql().unwrap();
        assert_eq!(sql, "SELECT * FROM users LEFT JOIN profiles ON users.id = profiles.user_id");
    }
    
    #[test]
    fn test_right_join() {
        let query = SelectBuilder::new("users")
            .right_join("orders", "users.id", "orders.user_id");
            
        let sql = query.to_sql().unwrap();
        assert_eq!(sql, "SELECT * FROM users RIGHT JOIN orders ON users.id = orders.user_id");
    }
    
    #[test]
    fn test_full_outer_join() {
        let query = SelectBuilder::new("users")
            .full_outer_join("profiles", "users.id", "profiles.user_id");
            
        let sql = query.to_sql().unwrap();
        assert_eq!(sql, "SELECT * FROM users FULL OUTER JOIN profiles ON users.id = profiles.user_id");
    }
    
    #[test]
    fn test_cross_join() {
        let query = SelectBuilder::new("users")
            .cross_join("categories");
            
        let sql = query.to_sql().unwrap();
        assert_eq!(sql, "SELECT * FROM users CROSS JOIN categories");
    }
    
    #[test]
    fn test_join_with_custom_operator() {
        let query = SelectBuilder::new("users")
            .join(JoinType::Inner, "profiles", "users.id", op::GT, "profiles.min_user_id");
            
        let sql = query.to_sql().unwrap();
        assert_eq!(sql, "SELECT * FROM users INNER JOIN profiles ON users.id > profiles.min_user_id");
    }
    
    #[test]
    fn test_multiple_joins() {
        let query = SelectBuilder::new("users")
            .inner_join("profiles", "users.id", "profiles.user_id")
            .left_join("orders", "users.id", "orders.user_id")
            .right_join("categories", "orders.category_id", "categories.id");
            
        let sql = query.to_sql().unwrap();
        assert_eq!(sql, "SELECT * FROM users INNER JOIN profiles ON users.id = profiles.user_id LEFT JOIN orders ON users.id = orders.user_id RIGHT JOIN categories ON orders.category_id = categories.id");
    }
    
    #[test]
    fn test_join_with_where_clause() {
        let query = SelectBuilder::new("users")
            .select(("users.name", "orders.total"))
            .inner_join("orders", "users.id", "orders.user_id")
            .where_(("users.active", true))
            .and_where(("orders.status", "completed"));
            
        let sql = query.to_sql().unwrap();
        assert_eq!(sql, "SELECT users.name, orders.total FROM users INNER JOIN orders ON users.id = orders.user_id WHERE users.active = ? AND orders.status = ?");
    }
    
    #[test]
    fn test_join_with_limit_offset() {
        let query = SelectBuilder::new("users")
            .inner_join("profiles", "users.id", "profiles.user_id")
            .limit(10)
            .offset(20);
            
        let sql = query.to_sql().unwrap();
        assert_eq!(sql, "SELECT * FROM users INNER JOIN profiles ON users.id = profiles.user_id LIMIT 10 OFFSET 20");
    }
    
    #[test]
    fn test_generic_join_method() {
        let query = SelectBuilder::new("users")
            .join(JoinType::Inner, "profiles", "users.id", op::EQ, "profiles.user_id");
            
        let sql = query.to_sql().unwrap();
        assert_eq!(sql, "SELECT * FROM users INNER JOIN profiles ON users.id = profiles.user_id");
    }
    
    // ORDER BY and GROUP BY tests
    #[test]
    fn test_order_by_asc() {
        let query = SelectBuilder::new("users")
            .order_by("name");
            
        let sql = query.to_sql().unwrap();
        assert_eq!(sql, "SELECT * FROM users ORDER BY name ASC");
    }
    
    #[test]
    fn test_order_by_desc() {
        let query = SelectBuilder::new("users")
            .order_by_desc("created_at");
            
        let sql = query.to_sql().unwrap();
        assert_eq!(sql, "SELECT * FROM users ORDER BY created_at DESC");
    }
    
    #[test]
    fn test_order_by_with_direction() {
        let query = SelectBuilder::new("users")
            .order_by_with_direction("age", SortDirection::Desc);
            
        let sql = query.to_sql().unwrap();
        assert_eq!(sql, "SELECT * FROM users ORDER BY age DESC");
    }
    
    #[test]
    fn test_multiple_order_by() {
        let query = SelectBuilder::new("users")
            .order_by("name")
            .order_by_desc("created_at")
            .order_by("id");
            
        let sql = query.to_sql().unwrap();
        assert_eq!(sql, "SELECT * FROM users ORDER BY name ASC, created_at DESC, id ASC");
    }
    
    #[test]
    fn test_group_by_single_column() {
        let query = SelectBuilder::new("orders")
            .select("status")
            .group_by("status");
            
        let sql = query.to_sql().unwrap();
        assert_eq!(sql, "SELECT status FROM orders GROUP BY status");
    }
    
    #[test]
    fn test_group_by_multiple_columns() {
        let query = SelectBuilder::new("orders")
            .select(("customer_id", "status"))
            .group_by(("customer_id", "status"));
            
        let sql = query.to_sql().unwrap();
        assert_eq!(sql, "SELECT customer_id, status FROM orders GROUP BY customer_id, status");
    }
    
    #[test]
    fn test_group_by_with_where() {
        let query = SelectBuilder::new("orders")
            .select("status")
            .where_(("active", true))
            .group_by("status");
            
        let sql = query.to_sql().unwrap();
        assert_eq!(sql, "SELECT status FROM orders WHERE active = ? GROUP BY status");
    }
    
    #[test]
    fn test_order_by_with_where() {
        let query = SelectBuilder::new("users")
            .where_(("active", true))
            .order_by("name");
            
        let sql = query.to_sql().unwrap();
        assert_eq!(sql, "SELECT * FROM users WHERE active = ? ORDER BY name ASC");
    }
    
    #[test]
    fn test_group_by_with_order_by() {
        let query = SelectBuilder::new("orders")
            .select("status")
            .group_by("status")
            .order_by("status");
            
        let sql = query.to_sql().unwrap();
        assert_eq!(sql, "SELECT status FROM orders GROUP BY status ORDER BY status ASC");
    }
    
    #[test]
    fn test_complex_query_with_joins_group_order() {
        let query = SelectBuilder::new("users")
            .select(("users.name", "orders.status"))
            .inner_join("orders", "users.id", "orders.user_id")
            .where_(("users.active", true))
            .group_by(("users.name", "orders.status"))
            .order_by("users.name")
            .order_by_desc("orders.status")
            .limit(10);
            
        let sql = query.to_sql().unwrap();
        assert_eq!(sql, "SELECT users.name, orders.status FROM users INNER JOIN orders ON users.id = orders.user_id WHERE users.active = ? GROUP BY users.name, orders.status ORDER BY users.name ASC, orders.status DESC LIMIT 10");
    }
    
    #[test]
    fn test_order_by_with_limit_offset() {
        let query = SelectBuilder::new("users")
            .order_by("created_at")
            .limit(25)
            .offset(50);
            
        let sql = query.to_sql().unwrap();
        assert_eq!(sql, "SELECT * FROM users ORDER BY created_at ASC LIMIT 25 OFFSET 50");
    }
    
    // DISTINCT operation tests
    #[test]
    fn test_distinct_basic() {
        let query = SelectBuilder::new("users")
            .select("status")
            .distinct();
            
        let sql = query.to_sql().unwrap();
        assert_eq!(sql, "SELECT DISTINCT status FROM users");
    }
    
    #[test]
    fn test_distinct_multiple_columns() {
        let query = SelectBuilder::new("users")
            .select(("status", "role"))
            .distinct();
            
        let sql = query.to_sql().unwrap();
        assert_eq!(sql, "SELECT DISTINCT status, role FROM users");
    }
    
    #[test]
    fn test_distinct_with_where() {
        let query = SelectBuilder::new("users")
            .select("department")
            .distinct()
            .where_(("active", true));
            
        let sql = query.to_sql().unwrap();
        assert_eq!(sql, "SELECT DISTINCT department FROM users WHERE active = ?");
    }
    
    #[test]
    fn test_distinct_with_join() {
        let query = SelectBuilder::new("users")
            .select("users.role")
            .distinct()
            .inner_join("departments", "users.dept_id", "departments.id");
            
        let sql = query.to_sql().unwrap();
        assert_eq!(sql, "SELECT DISTINCT users.role FROM users INNER JOIN departments ON users.dept_id = departments.id");
    }
    
    #[test]
    fn test_distinct_with_order_by() {
        let query = SelectBuilder::new("users")
            .select("status")
            .distinct()
            .order_by("status");
            
        let sql = query.to_sql().unwrap();
        assert_eq!(sql, "SELECT DISTINCT status FROM users ORDER BY status ASC");
    }
    
    #[test]
    fn test_distinct_with_group_by() {
        let query = SelectBuilder::new("orders")
            .select("customer_id")
            .distinct()
            .group_by("customer_id");
            
        let sql = query.to_sql().unwrap();
        assert_eq!(sql, "SELECT DISTINCT customer_id FROM orders GROUP BY customer_id");
    }
    
    #[test]
    fn test_distinct_with_limit() {
        let query = SelectBuilder::new("users")
            .select("department")
            .distinct()
            .limit(5);
            
        let sql = query.to_sql().unwrap();
        assert_eq!(sql, "SELECT DISTINCT department FROM users LIMIT 5");
    }
    
    #[test]
    fn test_complex_distinct_query() {
        let query = SelectBuilder::new("users")
            .select(("users.department", "roles.name"))
            .distinct()
            .inner_join("user_roles", "users.id", "user_roles.user_id")
            .inner_join("roles", "user_roles.role_id", "roles.id")
            .where_(("users.active", true))
            .and_where(("roles.active", true))
            .order_by("users.department")
            .order_by("roles.name")
            .limit(20);
            
        let sql = query.to_sql().unwrap();
        assert_eq!(sql, "SELECT DISTINCT users.department, roles.name FROM users INNER JOIN user_roles ON users.id = user_roles.user_id INNER JOIN roles ON user_roles.role_id = roles.id WHERE users.active = ? AND roles.active = ? ORDER BY users.department ASC, roles.name ASC LIMIT 20");
    }
    
    #[test]
    fn test_distinct_all_columns() {
        let query = SelectBuilder::new("users")
            .distinct();
            
        let sql = query.to_sql().unwrap();
        assert_eq!(sql, "SELECT DISTINCT * FROM users");
    }
    
    // Aggregation function tests
    #[test]
    fn test_count_all() {
        let query = SelectBuilder::new("users")
            .select(ColumnSelector::count());
            
        let sql = query.to_sql().unwrap();
        assert_eq!(sql, "SELECT COUNT(*) FROM users");
    }
    
    #[test]
    fn test_count_all_with_alias() {
        let query = SelectBuilder::new("users")
            .select(ColumnSelector::count_as("total"));
            
        let sql = query.to_sql().unwrap();
        assert_eq!(sql, "SELECT COUNT(*) AS total FROM users");
    }
    
    #[test]
    fn test_count_column() {
        let query = SelectBuilder::new("users")
            .select(ColumnSelector::count_column("id"));
            
        let sql = query.to_sql().unwrap();
        assert_eq!(sql, "SELECT COUNT(id) FROM users");
    }
    
    #[test]
    fn test_count_distinct() {
        let query = SelectBuilder::new("users")
            .select(ColumnSelector::count_distinct("email"));
            
        let sql = query.to_sql().unwrap();
        assert_eq!(sql, "SELECT COUNT(DISTINCT email) FROM users");
    }
    
    #[test]
    fn test_sum_function() {
        let query = SelectBuilder::new("orders")
            .select(ColumnSelector::sum("total"));
            
        let sql = query.to_sql().unwrap();
        assert_eq!(sql, "SELECT SUM(total) FROM orders");
    }
    
    #[test]
    fn test_avg_function() {
        let query = SelectBuilder::new("products")
            .select(ColumnSelector::avg("price"));
            
        let sql = query.to_sql().unwrap();
        assert_eq!(sql, "SELECT AVG(price) FROM products");
    }
    
    #[test]
    fn test_min_function() {
        let query = SelectBuilder::new("products")
            .select(ColumnSelector::min("price"));
            
        let sql = query.to_sql().unwrap();
        assert_eq!(sql, "SELECT MIN(price) FROM products");
    }
    
    #[test]
    fn test_max_function() {
        let query = SelectBuilder::new("products")
            .select(ColumnSelector::max("price"));
            
        let sql = query.to_sql().unwrap();
        assert_eq!(sql, "SELECT MAX(price) FROM products");
    }
    
    #[test]
    fn test_aggregation_with_alias() {
        let query = SelectBuilder::new("orders")
            .select(ColumnSelector::sum("total").as_alias("total_sales"));
            
        let sql = query.to_sql().unwrap();
        assert_eq!(sql, "SELECT SUM(total) AS total_sales FROM orders");
    }
    
    #[test]
    fn test_mixed_columns_and_aggregations() {
        let query = SelectBuilder::new("orders")
            .select(vec![
                ColumnSelector::Column("status".to_string()),
                ColumnSelector::count().as_alias("count"),
                ColumnSelector::sum("total").as_alias("total_sales")
            ]);
            
        let sql = query.to_sql().unwrap();
        assert_eq!(sql, "SELECT status, COUNT(*) AS count, SUM(total) AS total_sales FROM orders");
    }
    
    #[test]
    fn test_aggregation_with_group_by() {
        let query = SelectBuilder::new("orders")
            .select(vec![
                ColumnSelector::Column("status".to_string()),
                ColumnSelector::count().as_alias("count"),
                ColumnSelector::avg("total").as_alias("avg_total")
            ])
            .group_by("status");
            
        let sql = query.to_sql().unwrap();
        assert_eq!(sql, "SELECT status, COUNT(*) AS count, AVG(total) AS avg_total FROM orders GROUP BY status");
    }
    
    #[test]
    fn test_aggregation_with_joins() {
        let query = SelectBuilder::new("users")
            .select(vec![
                ColumnSelector::Column("users.name".to_string()),
                ColumnSelector::count().as_alias("order_count")
            ])
            .left_join("orders", "users.id", "orders.user_id")
            .group_by("users.name");
            
        let sql = query.to_sql().unwrap();
        assert_eq!(sql, "SELECT users.name, COUNT(*) AS order_count FROM users LEFT JOIN orders ON users.id = orders.user_id GROUP BY users.name");
    }
    
    #[test]
    fn test_complex_aggregation_query() {
        let query = SelectBuilder::new("orders")
            .select(vec![
                ColumnSelector::Column("customer_id".to_string()),
                ColumnSelector::Column("status".to_string()),
                ColumnSelector::count().as_alias("order_count"),
                ColumnSelector::sum("total").as_alias("total_sales"),
                ColumnSelector::avg("total").as_alias("avg_order_value"),
                ColumnSelector::min("total").as_alias("min_order"),
                ColumnSelector::max("total").as_alias("max_order")
            ])
            .where_(("status", "completed"))
            .group_by(("customer_id", "status"))
            .order_by("customer_id")
            .order_by_desc("total_sales")
            .limit(100);
            
        let sql = query.to_sql().unwrap();
        assert_eq!(sql, "SELECT customer_id, status, COUNT(*) AS order_count, SUM(total) AS total_sales, AVG(total) AS avg_order_value, MIN(total) AS min_order, MAX(total) AS max_order FROM orders WHERE status = ? GROUP BY customer_id, status ORDER BY customer_id ASC, total_sales DESC LIMIT 100");
    }
    
    // HAVING clause tests
    #[test]
    fn test_having_basic() {
        let query = SelectBuilder::new("orders")
            .select(vec![
                ColumnSelector::Column("status".to_string()),
                ColumnSelector::count().as_alias("count")
            ])
            .group_by("status")
            .having(("COUNT(*)", op::GT, 5));
            
        let sql = query.to_sql().unwrap();
        assert_eq!(sql, "SELECT status, COUNT(*) AS count FROM orders GROUP BY status HAVING COUNT(*) > ?");
    }
    
    #[test]
    fn test_having_with_sum() {
        let query = SelectBuilder::new("sales")
            .select(vec![
                ColumnSelector::Column("region".to_string()),
                ColumnSelector::sum("amount").as_alias("total_sales")
            ])
            .group_by("region")
            .having(("SUM(amount)", op::GTE, 10000));
            
        let sql = query.to_sql().unwrap();
        assert_eq!(sql, "SELECT region, SUM(amount) AS total_sales FROM sales GROUP BY region HAVING SUM(amount) >= ?");
    }
    
    #[test]
    fn test_having_with_avg() {
        let query = SelectBuilder::new("products")
            .select(vec![
                ColumnSelector::Column("category".to_string()),
                ColumnSelector::avg("price").as_alias("avg_price")
            ])
            .group_by("category")
            .having(("AVG(price)", op::LT, 100.0));
            
        let sql = query.to_sql().unwrap();
        assert_eq!(sql, "SELECT category, AVG(price) AS avg_price FROM products GROUP BY category HAVING AVG(price) < ?");
    }
    
    #[test]
    fn test_multiple_having_conditions() {
        let query = SelectBuilder::new("orders")
            .select(vec![
                ColumnSelector::Column("customer_id".to_string()),
                ColumnSelector::count().as_alias("order_count"),
                ColumnSelector::sum("total").as_alias("total_spent")
            ])
            .group_by("customer_id")
            .having(("COUNT(*)", op::GT, 3))
            .and_having(("SUM(total)", op::GTE, 500));
            
        let sql = query.to_sql().unwrap();
        assert_eq!(sql, "SELECT customer_id, COUNT(*) AS order_count, SUM(total) AS total_spent FROM orders GROUP BY customer_id HAVING COUNT(*) > ? AND SUM(total) >= ?");
    }
    
    #[test]
    fn test_having_with_or_condition() {
        let query = SelectBuilder::new("products")
            .select(vec![
                ColumnSelector::Column("category".to_string()),
                ColumnSelector::count().as_alias("product_count"),
                ColumnSelector::avg("price").as_alias("avg_price")
            ])
            .group_by("category")
            .having(("COUNT(*)", op::GT, 10))
            .or_having(("AVG(price)", op::LT, 50));
            
        let sql = query.to_sql().unwrap();
        assert_eq!(sql, "SELECT category, COUNT(*) AS product_count, AVG(price) AS avg_price FROM products GROUP BY category HAVING COUNT(*) > ? OR AVG(price) < ?");
    }
    
    #[test]
    fn test_having_with_where_and_group_by() {
        let query = SelectBuilder::new("orders")
            .select(vec![
                ColumnSelector::Column("status".to_string()),
                ColumnSelector::count().as_alias("count"),
                ColumnSelector::sum("total").as_alias("total_sales")
            ])
            .where_(("created_at", op::GTE, "2023-01-01"))
            .group_by("status")
            .having(("COUNT(*)", op::GT, 5));
            
        let sql = query.to_sql().unwrap();
        assert_eq!(sql, "SELECT status, COUNT(*) AS count, SUM(total) AS total_sales FROM orders WHERE created_at >= ? GROUP BY status HAVING COUNT(*) > ?");
    }
    
    #[test]
    fn test_having_with_joins() {
        let query = SelectBuilder::new("users")
            .select(vec![
                ColumnSelector::Column("users.department".to_string()),
                ColumnSelector::count().as_alias("user_count"),
                ColumnSelector::avg("salaries.amount").as_alias("avg_salary")
            ])
            .inner_join("salaries", "users.id", "salaries.user_id")
            .group_by("users.department")
            .having(("COUNT(*)", op::GTE, 5))
            .and_having(("AVG(salaries.amount)", op::GT, 75000));
            
        let sql = query.to_sql().unwrap();
        assert_eq!(sql, "SELECT users.department, COUNT(*) AS user_count, AVG(salaries.amount) AS avg_salary FROM users INNER JOIN salaries ON users.id = salaries.user_id GROUP BY users.department HAVING COUNT(*) >= ? AND AVG(salaries.amount) > ?");
    }
    
    #[test]
    fn test_having_with_order_by() {
        let query = SelectBuilder::new("products")
            .select(vec![
                ColumnSelector::Column("category".to_string()),
                ColumnSelector::count().as_alias("product_count"),
                ColumnSelector::max("price").as_alias("max_price")
            ])
            .group_by("category")
            .having(("COUNT(*)", op::GT, 5))
            .order_by("product_count")
            .order_by_desc("max_price");
            
        let sql = query.to_sql().unwrap();
        assert_eq!(sql, "SELECT category, COUNT(*) AS product_count, MAX(price) AS max_price FROM products GROUP BY category HAVING COUNT(*) > ? ORDER BY product_count ASC, max_price DESC");
    }
    
    #[test]
    fn test_complex_having_query() {
        let query = SelectBuilder::new("sales")
            .select(vec![
                ColumnSelector::Column("region".to_string()),
                ColumnSelector::Column("quarter".to_string()),
                ColumnSelector::count().as_alias("sale_count"),
                ColumnSelector::sum("amount").as_alias("total_sales"),
                ColumnSelector::avg("amount").as_alias("avg_sale"),
                ColumnSelector::min("amount").as_alias("min_sale"),
                ColumnSelector::max("amount").as_alias("max_sale")
            ])
            .inner_join("products", "sales.product_id", "products.id")
            .where_(("sales.date", op::GTE, "2023-01-01"))
            .and_where(("products.active", true))
            .group_by(("region", "quarter"))
            .having(("COUNT(*)", op::GT, 10))
            .and_having(("SUM(amount)", op::GTE, 50000))
            .or_having(("AVG(amount)", op::GT, 1000))
            .order_by("region")
            .order_by("quarter")
            .order_by_desc("total_sales")
            .limit(20);
            
        let sql = query.to_sql().unwrap();
        assert_eq!(sql, "SELECT region, quarter, COUNT(*) AS sale_count, SUM(amount) AS total_sales, AVG(amount) AS avg_sale, MIN(amount) AS min_sale, MAX(amount) AS max_sale FROM sales INNER JOIN products ON sales.product_id = products.id WHERE sales.date >= ? AND products.active = ? GROUP BY region, quarter HAVING COUNT(*) > ? AND SUM(amount) >= ? OR AVG(amount) > ? ORDER BY region ASC, quarter ASC, total_sales DESC LIMIT 20");
    }
    
    #[test]
    fn test_having_count_distinct() {
        let query = SelectBuilder::new("orders")
            .select(vec![
                ColumnSelector::Column("region".to_string()),
                ColumnSelector::count_distinct("customer_id").as_alias("unique_customers"),
                ColumnSelector::sum("total").as_alias("total_sales")
            ])
            .group_by("region")
            .having(("COUNT(DISTINCT customer_id)", op::GT, 100));
            
        let sql = query.to_sql().unwrap();
        assert_eq!(sql, "SELECT region, COUNT(DISTINCT customer_id) AS unique_customers, SUM(total) AS total_sales FROM orders GROUP BY region HAVING COUNT(DISTINCT customer_id) > ?");
    }
}