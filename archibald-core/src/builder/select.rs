//! SELECT query builder implementation

use crate::{Result, Error, Value, IntoOperator};
use super::common::{
    QueryBuilder, IntoCondition, WhereCondition, WhereConnector, 
    AggregateFunction, IntoColumns, IntoColumnSelectors, JoinType, JoinConnector, JoinClause,
    SortDirection, OrderByClause, GroupByClause, HavingCondition
};

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
    SubqueryColumn {
        subquery: Subquery,
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

    /// Add alias to this column selector
    pub fn as_alias(mut self, alias: &str) -> Self {
        match self {
            Self::Column(_) => {
                // For regular columns, we can't easily add an alias to the current enum variant
                // This would require restructuring the enum. For now, leave as-is
                self
            },
            Self::Aggregate { alias: ref mut alias_field, .. } => {
                *alias_field = Some(alias.to_string());
                self
            },
            Self::CountAll { alias: ref mut alias_field } => {
                *alias_field = Some(alias.to_string());
                self
            },
            Self::SubqueryColumn { alias: ref mut alias_field, .. } => {
                *alias_field = Some(alias.to_string());
                self
            },
        }
    }

    /// Create a subquery column selector with alias
    pub fn subquery_as(query: SelectBuilderComplete, alias: &str) -> Self {
        Self::SubqueryColumn {
            subquery: Subquery::new(query),
            alias: Some(alias.to_string()),
        }
    }
}

/// Subquery wrapper for use in various SQL contexts
#[derive(Debug, Clone)]
pub struct Subquery {
    pub query: Box<SelectBuilderComplete>,
}

impl Subquery {
    /// Create a new subquery from a SelectBuilder
    pub fn new(query: SelectBuilderComplete) -> Self {
        Self {
            query: Box::new(query),
        }
    }

    /// Convert to SQL string
    pub fn to_sql(&self) -> Result<String> {
        let inner_sql = self.query.to_sql()?;
        Ok(format!("({})", inner_sql))
    }

    /// Get parameters from the subquery
    pub fn parameters(&self) -> &[Value] {
        self.query.parameters()
    }
}

/// A subquery condition for WHERE IN, WHERE EXISTS, etc
#[derive(Debug, Clone)]
pub struct SubqueryCondition {
    pub column: String,
    pub operator: crate::Operator,
    pub subquery: Subquery,
    pub connector: WhereConnector,
}

/// SELECT query builder in initial state (before select() is called)
/// Can build conditions but cannot execute queries  
#[derive(Debug, Clone)]
pub struct SelectBuilderInitial {
    table_name: String,
    where_conditions: Vec<WhereCondition>,
    subquery_conditions: Vec<SubqueryCondition>,
    join_clauses: Vec<JoinClause>,
    order_by_clauses: Vec<OrderByClause>,
    group_by_clause: Option<GroupByClause>,
    having_conditions: Vec<HavingCondition>,
    distinct: bool,
    limit_value: Option<u64>,
    offset_value: Option<u64>,
    parameters: Vec<Value>,
}

/// SELECT query builder in complete state (after select() is called)
/// Can execute queries and add more conditions
#[derive(Debug, Clone)]
pub struct SelectBuilderComplete {
    pub table_name: String,
    pub selected_columns: Vec<ColumnSelector>,
    pub where_conditions: Vec<WhereCondition>,
    pub subquery_conditions: Vec<SubqueryCondition>,
    pub join_clauses: Vec<JoinClause>,
    pub order_by_clauses: Vec<OrderByClause>,
    pub group_by_clause: Option<GroupByClause>,
    pub having_conditions: Vec<HavingCondition>,
    pub distinct: bool,
    pub limit_value: Option<u64>,
    pub offset_value: Option<u64>,
    pub parameters: Vec<Value>,
}

impl SelectBuilderInitial {
    /// Create a new SELECT query builder in initial state
    pub fn new(table: &str) -> Self {
        Self {
            table_name: table.to_string(),
            where_conditions: Vec::new(),
            subquery_conditions: Vec::new(),
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

    /// Select specific columns, transitioning to SelectBuilderComplete
    ///
    /// # Examples
    /// ```
    /// use archibald_core::from;
    ///
    /// let query = from("users").select(("id", "name", "email"));
    /// ```
    pub fn select<T>(self, columns: T) -> SelectBuilderComplete
    where
        T: IntoColumnSelectors,
    {
        let selected_columns = columns.into_column_selectors();

        SelectBuilderComplete {
            table_name: self.table_name,
            selected_columns,
            where_conditions: self.where_conditions,
            subquery_conditions: self.subquery_conditions,
            join_clauses: self.join_clauses,
            order_by_clauses: self.order_by_clauses,
            group_by_clause: self.group_by_clause,
            having_conditions: self.having_conditions,
            distinct: self.distinct,
            limit_value: self.limit_value,
            offset_value: self.offset_value,
            parameters: self.parameters,
        }
    }

    /// Select all columns, transitioning to SelectBuilderComplete
    pub fn select_all(self) -> SelectBuilderComplete {
        SelectBuilderComplete {
            table_name: self.table_name,
            selected_columns: vec![ColumnSelector::Column("*".to_string())],
            where_conditions: self.where_conditions,
            subquery_conditions: self.subquery_conditions,
            join_clauses: self.join_clauses,
            order_by_clauses: self.order_by_clauses,
            group_by_clause: self.group_by_clause,
            having_conditions: self.having_conditions,
            distinct: self.distinct,
            limit_value: self.limit_value,
            offset_value: self.offset_value,
            parameters: self.parameters,
        }
    }

    /// Add a WHERE condition
    ///
    /// # Examples
    /// ```
    /// use archibald_core::{from, op};
    ///
    /// let query = from("users")
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
        self.parameters.push(self.where_conditions.last().unwrap().value.clone());

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
        self.parameters.push(self.where_conditions.last().unwrap().value.clone());

        self
    }

    /// Add an AND WHERE condition (same as where_)
    pub fn and_where<C>(self, condition: C) -> Self
    where
        C: IntoCondition,
    {
        self.where_(condition)
    }

    /// Add a WHERE IN condition with a subquery
    ///
    /// # Examples
    /// ```
    /// use archibald_core::from;
    ///
    /// let subquery = from("orders").select("customer_id").where_(("status", "active"));
    /// let query = from("customers").where_in("id", subquery);
    /// ```
    pub fn where_in(mut self, column: &str, subquery: SelectBuilderComplete) -> Self {
        self.subquery_conditions.push(SubqueryCondition {
            column: column.to_string(),
            operator: crate::Operator::IN,
            subquery: Subquery::new(subquery),
            connector: WhereConnector::And,
        });
        self
    }

    /// Add a WHERE EXISTS condition with a subquery
    pub fn where_exists(mut self, subquery: SelectBuilderComplete) -> Self {
        self.subquery_conditions.push(SubqueryCondition {
            column: "".to_string(), // EXISTS doesn't need a column
            operator: crate::Operator::EXISTS,
            subquery: Subquery::new(subquery),
            connector: WhereConnector::And,
        });
        self
    }

    /// Add a WHERE NOT IN condition with a subquery
    pub fn where_not_in(mut self, column: &str, subquery: SelectBuilderComplete) -> Self {
        self.subquery_conditions.push(SubqueryCondition {
            column: column.to_string(),
            operator: crate::Operator::NOT_IN,
            subquery: Subquery::new(subquery),
            connector: WhereConnector::And,
        });
        self
    }

    /// Add a WHERE NOT EXISTS condition with a subquery
    pub fn where_not_exists(mut self, subquery: SelectBuilderComplete) -> Self {
        self.subquery_conditions.push(SubqueryCondition {
            column: "".to_string(), // NOT EXISTS doesn't need a column
            operator: crate::Operator::NOT_EXISTS,
            subquery: Subquery::new(subquery),
            connector: WhereConnector::And,
        });
        self
    }

    /// Add an INNER JOIN clause
    ///
    /// # Examples
    /// ```
    /// use archibald_core::from;
    ///
    /// let query = from("users")
    ///     .inner_join("posts", "users.id", "posts.user_id");
    /// ```
    pub fn inner_join(mut self, table: &str, left_column: &str, right_column: &str) -> Self {
        self.join_clauses.push(JoinClause {
            join_type: JoinType::Inner,
            table: table.to_string(),
            on_conditions: vec![super::common::JoinCondition {
                left_column: left_column.to_string(),
                operator: crate::Operator::EQ,
                right_column: right_column.to_string(),
                connector: JoinConnector::And,
            }],
        });
        self
    }

    /// Add a LEFT JOIN clause
    pub fn left_join(mut self, table: &str, left_column: &str, right_column: &str) -> Self {
        self.join_clauses.push(JoinClause {
            join_type: JoinType::Left,
            table: table.to_string(),
            on_conditions: vec![super::common::JoinCondition {
                left_column: left_column.to_string(),
                operator: crate::Operator::EQ,
                right_column: right_column.to_string(),
                connector: JoinConnector::And,
            }],
        });
        self
    }

    /// Add a RIGHT JOIN clause
    pub fn right_join(mut self, table: &str, left_column: &str, right_column: &str) -> Self {
        self.join_clauses.push(JoinClause {
            join_type: JoinType::Right,
            table: table.to_string(),
            on_conditions: vec![super::common::JoinCondition {
                left_column: left_column.to_string(),
                operator: crate::Operator::EQ,
                right_column: right_column.to_string(),
                connector: JoinConnector::And,
            }],
        });
        self
    }

    /// Add a FULL OUTER JOIN clause
    pub fn full_outer_join(mut self, table: &str, left_column: &str, right_column: &str) -> Self {
        self.join_clauses.push(JoinClause {
            join_type: JoinType::Full,
            table: table.to_string(),
            on_conditions: vec![super::common::JoinCondition {
                left_column: left_column.to_string(),
                operator: crate::Operator::EQ,
                right_column: right_column.to_string(),
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
    /// use archibald_core::{from, JoinType, op};
    ///
    /// let query = from("users")
    ///     .join(JoinType::Left, "profiles", "users.id", op::EQ, "profiles.user_id");
    /// ```
    pub fn join<O>(mut self, join_type: JoinType, table: &str, left_col: &str, operator: O, right_col: &str) -> Self
    where
        O: IntoOperator,
    {
        self.join_clauses.push(JoinClause {
            join_type,
            table: table.to_string(),
            on_conditions: vec![super::common::JoinCondition {
                left_column: left_col.to_string(),
                operator: operator.into_operator(),
                right_column: right_col.to_string(),
                connector: JoinConnector::And,
            }],
        });
        self
    }

    /// Add a GROUP BY clause
    ///
    /// # Examples
    /// ```
    /// use archibald_core::from;
    ///
    /// let query = from("orders").group_by(("customer_id", "status"));
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

    /// Add a HAVING condition (requires GROUP BY)
    ///
    /// # Examples
    /// ```
    /// use archibald_core::{from, ColumnSelector, op};
    ///
    /// let query = from("orders")
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
            value: value.clone(),
            connector: WhereConnector::And,
        });
        self.parameters.push(value);
        self
    }

    /// Add an AND HAVING condition (requires GROUP BY)
    pub fn and_having<C>(mut self, condition: C) -> Self
    where
        C: IntoCondition,
    {
        let (column, operator, value) = condition.into_condition();
        self.having_conditions.push(HavingCondition {
            column_or_function: column,
            operator,
            value: value.clone(),
            connector: WhereConnector::And,
        });
        self.parameters.push(value);
        self
    }

    /// Add an OR HAVING condition (requires GROUP BY)
    pub fn or_having<C>(mut self, condition: C) -> Self
    where
        C: IntoCondition,
    {
        let (column, operator, value) = condition.into_condition();
        self.having_conditions.push(HavingCondition {
            column_or_function: column,
            operator,
            value: value.clone(),
            connector: WhereConnector::Or,
        });
        self.parameters.push(value);
        self
    }

    /// Add an ORDER BY clause
    ///
    /// # Examples
    /// ```
    /// use archibald_core::{from, SortDirection};
    ///
    /// let query = from("users").order_by("name", SortDirection::Asc);
    /// ```
    pub fn order_by(mut self, column: &str, direction: SortDirection) -> Self {
        self.order_by_clauses.push(OrderByClause {
            column: column.to_string(),
            direction,
        });
        self
    }

    /// Add an ORDER BY ASC clause (convenience method)
    ///
    /// # Examples
    /// ```
    /// use archibald_core::from;
    ///
    /// let query = from("users").order_by_asc("created_at");
    /// ```
    pub fn order_by_asc(mut self, column: &str) -> Self {
        self.order_by_clauses.push(OrderByClause {
            column: column.to_string(),
            direction: SortDirection::Asc,
        });
        self
    }

    /// Add an ORDER BY DESC clause (convenience method)
    ///
    /// # Examples
    /// ```
    /// use archibald_core::from;
    ///
    /// let query = from("users").order_by_desc("created_at");
    /// ```
    pub fn order_by_desc(mut self, column: &str) -> Self {
        self.order_by_clauses.push(OrderByClause {
            column: column.to_string(),
            direction: SortDirection::Desc,
        });
        self
    }

    /// Add a LIMIT clause
    pub fn limit(mut self, count: u64) -> Self {
        self.limit_value = Some(count);
        self
    }

    /// Add an OFFSET clause
    pub fn offset(mut self, offset: u64) -> Self {
        self.offset_value = Some(offset);
        self
    }
}

impl SelectBuilderComplete {
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
        self.parameters.push(self.where_conditions.last().unwrap().value.clone());

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
        self.parameters.push(self.where_conditions.last().unwrap().value.clone());

        self
    }

    /// Add an AND WHERE condition (same as where_)
    pub fn and_where<C>(self, condition: C) -> Self
    where
        C: IntoCondition,
    {
        self.where_(condition)
    }

    /// Add an ORDER BY clause
    pub fn order_by(mut self, column: &str, direction: SortDirection) -> Self {
        self.order_by_clauses.push(OrderByClause {
            column: column.to_string(),
            direction,
        });
        self
    }

    /// Add an ORDER BY ASC clause (convenience method)
    pub fn order_by_asc(self, column: &str) -> Self {
        self.order_by(column, SortDirection::Asc)
    }

    /// Add an ORDER BY DESC clause (convenience method)
    pub fn order_by_desc(self, column: &str) -> Self {
        self.order_by(column, SortDirection::Desc)
    }

    /// Add a LIMIT clause
    pub fn limit(mut self, count: u64) -> Self {
        self.limit_value = Some(count);
        self
    }

    /// Add an OFFSET clause
    pub fn offset(mut self, offset: u64) -> Self {
        self.offset_value = Some(offset);
        self
    }

    /// Mark the query as DISTINCT
    ///
    /// # Examples
    /// ```
    /// use archibald_core::from;
    ///
    /// let query = from("users").select("status").distinct();
    /// ```
    pub fn distinct(mut self) -> Self {
        self.distinct = true;
        self
    }

    /// Add a WHERE IN condition with a subquery
    pub fn where_in(mut self, column: &str, subquery: SelectBuilderComplete) -> Self {
        self.subquery_conditions.push(SubqueryCondition {
            column: column.to_string(),
            operator: crate::Operator::IN,
            subquery: Subquery::new(subquery),
            connector: WhereConnector::And,
        });
        // Parameters from subqueries are handled inside the Subquery struct
        self
    }

    /// Add a WHERE EXISTS condition with a subquery
    pub fn where_exists(mut self, subquery: SelectBuilderComplete) -> Self {
        self.subquery_conditions.push(SubqueryCondition {
            column: "".to_string(), // EXISTS doesn't need a column
            operator: crate::Operator::EXISTS,
            subquery: Subquery::new(subquery),
            connector: WhereConnector::And,
        });
        // Parameters from subqueries are handled inside the Subquery struct
        self
    }

    /// Add a WHERE NOT IN condition with a subquery
    pub fn where_not_in(mut self, column: &str, subquery: SelectBuilderComplete) -> Self {
        self.subquery_conditions.push(SubqueryCondition {
            column: column.to_string(),
            operator: crate::Operator::NOT_IN,
            subquery: Subquery::new(subquery),
            connector: WhereConnector::And,
        });
        // Parameters from subqueries are handled inside the Subquery struct
        self
    }

    /// Add a WHERE NOT EXISTS condition with a subquery
    pub fn where_not_exists(mut self, subquery: SelectBuilderComplete) -> Self {
        self.subquery_conditions.push(SubqueryCondition {
            column: "".to_string(), // NOT EXISTS doesn't need a column
            operator: crate::Operator::NOT_EXISTS,
            subquery: Subquery::new(subquery),
            connector: WhereConnector::And,
        });
        // Parameters from subqueries are handled inside the Subquery struct
        self
    }

    /// Add an INNER JOIN clause
    pub fn inner_join(mut self, table: &str, left_column: &str, right_column: &str) -> Self {
        self.join_clauses.push(JoinClause {
            join_type: JoinType::Inner,
            table: table.to_string(),
            on_conditions: vec![super::common::JoinCondition {
                left_column: left_column.to_string(),
                operator: crate::Operator::EQ,
                right_column: right_column.to_string(),
                connector: JoinConnector::And,
            }],
        });
        self
    }

    /// Add a LEFT JOIN clause
    pub fn left_join(mut self, table: &str, left_column: &str, right_column: &str) -> Self {
        self.join_clauses.push(JoinClause {
            join_type: JoinType::Left,
            table: table.to_string(),
            on_conditions: vec![super::common::JoinCondition {
                left_column: left_column.to_string(),
                operator: crate::Operator::EQ,
                right_column: right_column.to_string(),
                connector: JoinConnector::And,
            }],
        });
        self
    }

    /// Add a RIGHT JOIN clause
    pub fn right_join(mut self, table: &str, left_column: &str, right_column: &str) -> Self {
        self.join_clauses.push(JoinClause {
            join_type: JoinType::Right,
            table: table.to_string(),
            on_conditions: vec![super::common::JoinCondition {
                left_column: left_column.to_string(),
                operator: crate::Operator::EQ,
                right_column: right_column.to_string(),
                connector: JoinConnector::And,
            }],
        });
        self
    }

    /// Add a FULL OUTER JOIN clause
    pub fn full_outer_join(mut self, table: &str, left_column: &str, right_column: &str) -> Self {
        self.join_clauses.push(JoinClause {
            join_type: JoinType::Full,
            table: table.to_string(),
            on_conditions: vec![super::common::JoinCondition {
                left_column: left_column.to_string(),
                operator: crate::Operator::EQ,
                right_column: right_column.to_string(),
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
    pub fn join<O>(mut self, join_type: JoinType, table: &str, left_col: &str, operator: O, right_col: &str) -> Self
    where
        O: IntoOperator,
    {
        self.join_clauses.push(JoinClause {
            join_type,
            table: table.to_string(),
            on_conditions: vec![super::common::JoinCondition {
                left_column: left_col.to_string(),
                operator: operator.into_operator(),
                right_column: right_col.to_string(),
                connector: JoinConnector::And,
            }],
        });
        self
    }

    /// Add a GROUP BY clause
    pub fn group_by<C>(mut self, columns: C) -> Self
    where
        C: IntoColumns,
    {
        self.group_by_clause = Some(GroupByClause {
            columns: columns.into_columns(),
        });
        self
    }

    /// Add a HAVING condition (requires GROUP BY)
    pub fn having<C>(mut self, condition: C) -> Self
    where
        C: IntoCondition,
    {
        let (column, operator, value) = condition.into_condition();
        self.having_conditions.push(HavingCondition {
            column_or_function: column,
            operator,
            value: value.clone(),
            connector: WhereConnector::And,
        });
        self.parameters.push(value);
        self
    }

    /// Add an AND HAVING condition (requires GROUP BY)
    pub fn and_having<C>(mut self, condition: C) -> Self
    where
        C: IntoCondition,
    {
        let (column, operator, value) = condition.into_condition();
        self.having_conditions.push(HavingCondition {
            column_or_function: column,
            operator,
            value: value.clone(),
            connector: WhereConnector::And,
        });
        self.parameters.push(value);
        self
    }

    /// Add an OR HAVING condition (requires GROUP BY)
    pub fn or_having<C>(mut self, condition: C) -> Self
    where
        C: IntoCondition,
    {
        let (column, operator, value) = condition.into_condition();
        self.having_conditions.push(HavingCondition {
            column_or_function: column,
            operator,
            value: value.clone(),
            connector: WhereConnector::Or,
        });
        self.parameters.push(value);
        self
    }
}

impl QueryBuilder for SelectBuilderInitial {
    fn to_sql(&self) -> Result<String> {
        Err(Error::invalid_query("SELECT requires columns to be specified with .select()"))
    }

    fn parameters(&self) -> &[Value] {
        &[]
    }

    fn clone_builder(&self) -> Self {
        self.clone()
    }
}

impl QueryBuilder for SelectBuilderComplete {
    fn to_sql(&self) -> Result<String> {
        // Validate all operators before generating SQL
        for condition in &self.where_conditions {
            condition.operator.validate()?;
        }

        for condition in &self.subquery_conditions {
            condition.operator.validate()?;
        }

        let mut sql = String::new();

        // SELECT clause
        sql.push_str("SELECT ");
        
        if self.distinct {
            sql.push_str("DISTINCT ");
        }

        // Columns
        if self.selected_columns.is_empty() {
            sql.push_str("*");
        } else {
            let mut column_parts = Vec::new();
            for col in &self.selected_columns {
                let part = match col {
                    ColumnSelector::Column(name) => name.clone(),
                    ColumnSelector::Aggregate { function, column, alias } => {
                        let func_sql = match function {
                            AggregateFunction::CountDistinct => {
                                format!("{}({}))", function, column)
                            }
                            _ => format!("{}({})", function, column),
                        };
                        if let Some(alias) = alias {
                            format!("{} AS {}", func_sql, alias)
                        } else {
                            func_sql
                        }
                    }
                    ColumnSelector::CountAll { alias } => {
                        let count_sql = "COUNT(*)".to_string();
                        if let Some(alias) = alias {
                            format!("{} AS {}", count_sql, alias)
                        } else {
                            count_sql
                        }
                    }
                    ColumnSelector::SubqueryColumn { subquery, alias } => {
                        let subquery_sql = subquery.to_sql()?;
                        if let Some(alias) = alias {
                            format!("{} AS {}", subquery_sql, alias)
                        } else {
                            subquery_sql
                        }
                    }
                };
                column_parts.push(part);
            }
            sql.push_str(&column_parts.join(", "));
        }

        // FROM clause
        sql.push_str(" FROM ");
        sql.push_str(&self.table_name);

        // JOIN clauses
        for join in &self.join_clauses {
            sql.push(' ');
            sql.push_str(&join.join_type.to_string());
            sql.push_str(" JOIN ");
            sql.push_str(&join.table);
            
            if !join.on_conditions.is_empty() {
                sql.push_str(" ON ");
                
                for (i, condition) in join.on_conditions.iter().enumerate() {
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
        if !self.where_conditions.is_empty() || !self.subquery_conditions.is_empty() {
            sql.push_str(" WHERE ");

            let mut conditions_added = 0;

            // Regular WHERE conditions
            for (i, condition) in self.where_conditions.iter().enumerate() {
                if conditions_added > 0 || i > 0 {
                    match condition.connector {
                        WhereConnector::And => sql.push_str(" AND "),
                        WhereConnector::Or => sql.push_str(" OR "),
                    }
                }

                sql.push_str(&condition.column);
                sql.push(' ');
                sql.push_str(condition.operator.as_str());
                sql.push_str(" ?");
                conditions_added += 1;
            }

            // Subquery conditions
            for condition in &self.subquery_conditions {
                if conditions_added > 0 {
                    match condition.connector {
                        WhereConnector::And => sql.push_str(" AND "),
                        WhereConnector::Or => sql.push_str(" OR "),
                    }
                }

                sql.push_str(&condition.column);
                if !condition.column.is_empty() {
                    sql.push(' ');
                }
                sql.push_str(condition.operator.as_str());
                sql.push(' ');
                sql.push_str(&condition.subquery.to_sql()?);
                conditions_added += 1;
            }
        }

        // GROUP BY clause
        if let Some(group_by) = &self.group_by_clause {
            sql.push_str(" GROUP BY ");
            sql.push_str(&group_by.columns.join(", "));

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
                    sql.push_str(" ?");
                }
            }
        }

        // ORDER BY clause
        if !self.order_by_clauses.is_empty() {
            sql.push_str(" ORDER BY ");
            let order_parts: Vec<String> = self.order_by_clauses
                .iter()
                .map(|clause| format!("{} {}", clause.column, clause.direction))
                .collect();
            sql.push_str(&order_parts.join(", "));
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::operator::op;
    use crate::from;

    #[test]
    fn test_basic_select() {
        let query = from("users").select("*");
        let sql = query.to_sql().unwrap();
        assert_eq!(sql, "SELECT * FROM users");
    }

    #[test]
    fn test_select_columns() {
        let query = from("users").select(("id", "name"));
        let sql = query.to_sql().unwrap();
        assert_eq!(sql, "SELECT id, name FROM users");
    }

    #[test]
    fn test_select_with_where() {
        let query = from("users").select("*").where_(("age", op::GT, 18));
        let sql = query.to_sql().unwrap();
        assert_eq!(sql, "SELECT * FROM users WHERE age > ?");
    }

    #[test]
    fn test_multiple_where_conditions() {
        let query = from("users")
            .select("*")
            .where_(("age", op::GT, 18))
            .where_(("name", "John"));
        let sql = query.to_sql().unwrap();
        assert_eq!(sql, "SELECT * FROM users WHERE age > ? AND name = ?");
    }

    #[test]
    fn test_or_where() {
        let query = from("users")
            .select("*")
            .where_(("age", op::GT, 18))
            .or_where(("status", "admin"));
        let sql = query.to_sql().unwrap();
        assert_eq!(sql, "SELECT * FROM users WHERE age > ? OR status = ?");
    }

    #[test]
    fn test_limit_and_offset() {
        let query = from("users")
            .select("*")
            .limit(10)
            .offset(5);
        let sql = query.to_sql().unwrap();
        assert_eq!(sql, "SELECT * FROM users LIMIT 10 OFFSET 5");
    }

    #[test]
    fn test_inner_join() {
        let query = from("users")
            .select("*")
            .inner_join("profiles", "users.id", "profiles.user_id");
        let sql = query.to_sql().unwrap();
        assert_eq!(sql, "SELECT * FROM users INNER JOIN profiles ON users.id = profiles.user_id");
    }

    #[test]
    fn test_left_join() {
        let query = from("users")
            .select("*")
            .left_join("profiles", "users.id", "profiles.user_id");
        let sql = query.to_sql().unwrap();
        assert_eq!(sql, "SELECT * FROM users LEFT JOIN profiles ON users.id = profiles.user_id");
    }

    #[test]
    fn test_right_join() {
        let query = from("users")
            .select("*")
            .right_join("profiles", "users.id", "profiles.user_id");
        let sql = query.to_sql().unwrap();
        assert_eq!(sql, "SELECT * FROM users RIGHT JOIN profiles ON users.id = profiles.user_id");
    }

    #[test]
    fn test_order_by_with_direction() {
        let query = from("users")
            .select("*")
            .order_by("name", SortDirection::Desc);
        let sql = query.to_sql().unwrap();
        assert_eq!(sql, "SELECT * FROM users ORDER BY name DESC");
    }

    #[test]
    fn test_order_by_asc() {
        let query = from("users")
            .select("*")
            .order_by_asc("name");
        let sql = query.to_sql().unwrap();
        assert_eq!(sql, "SELECT * FROM users ORDER BY name ASC");
    }

    #[test]
    fn test_order_by_desc() {
        let query = from("users")
            .select("*")
            .order_by_desc("created_at");
        let sql = query.to_sql().unwrap();
        assert_eq!(sql, "SELECT * FROM users ORDER BY created_at DESC");
    }

    #[test]
    fn test_group_by_single_column() {
        let query = from("users")
            .select("*")
            .group_by("department");
        let sql = query.to_sql().unwrap();
        assert_eq!(sql, "SELECT * FROM users GROUP BY department");
    }

    #[test]
    fn test_group_by_multiple_columns() {
        let query = from("users")
            .select("*")
            .group_by(("department", "status"));
        let sql = query.to_sql().unwrap();
        assert_eq!(sql, "SELECT * FROM users GROUP BY department, status");
    }

    #[test]
    fn test_distinct_basic() {
        let query = from("users")
            .select("status")
            .distinct();
        let sql = query.to_sql().unwrap();
        assert_eq!(sql, "SELECT DISTINCT status FROM users");
    }

    #[test]
    fn test_having_basic() {
        let query = from("users")
            .select("*")
            .group_by("department")
            .having(("COUNT(*)", op::GT, 5));
        let sql = query.to_sql().unwrap();
        assert_eq!(sql, "SELECT * FROM users GROUP BY department HAVING COUNT(*) > ?");
    }

    #[test]
    fn test_avg_function() {
        let query = from("products")
            .select(ColumnSelector::avg("price"));

        let sql = query.to_sql().unwrap();
        assert_eq!(sql, "SELECT AVG(price) FROM products");
    }

    #[test]
    fn test_min_function() {
        let query = from("products")
            .select(ColumnSelector::min("price"));

        let sql = query.to_sql().unwrap();
        assert_eq!(sql, "SELECT MIN(price) FROM products");
    }

    #[test]
    fn test_max_function() {
        let query = from("products")
            .select(ColumnSelector::max("price"));

        let sql = query.to_sql().unwrap();
        assert_eq!(sql, "SELECT MAX(price) FROM products");
    }

    #[test]
    fn test_sum_function() {
        let query = from("orders")
            .select(ColumnSelector::sum("total"));

        let sql = query.to_sql().unwrap();
        assert_eq!(sql, "SELECT SUM(total) FROM orders");
    }

    #[test]
    fn test_aggregation_with_alias() {
        let query = from("orders")
            .select(ColumnSelector::sum("total").as_alias("total_sales"));

        let sql = query.to_sql().unwrap();
        assert_eq!(sql, "SELECT SUM(total) AS total_sales FROM orders");
    }

    #[test]
    fn test_count_all() {
        let query = from("users")
            .select(ColumnSelector::count());

        let sql = query.to_sql().unwrap();
        assert_eq!(sql, "SELECT COUNT(*) FROM users");
    }

    #[test]
    fn test_count_all_with_alias() {
        let query = from("users")
            .select(ColumnSelector::count_as("total_users"));

        let sql = query.to_sql().unwrap();
        assert_eq!(sql, "SELECT COUNT(*) AS total_users FROM users");
    }

    #[test]
    fn test_count_column() {
        let query = from("users")
            .select(ColumnSelector::count_column("email"));

        let sql = query.to_sql().unwrap();
        assert_eq!(sql, "SELECT COUNT(email) FROM users");
    }

    #[test]
    fn test_count_distinct() {
        let query = from("orders")
            .select(ColumnSelector::count_distinct("customer_id"));

        let sql = query.to_sql().unwrap();
        assert_eq!(sql, "SELECT COUNT(DISTINCT(customer_id)) FROM orders");
    }

    #[test]
    fn test_cross_join() {
        let query = from("users")
            .select("*")
            .cross_join("categories");
        
        let sql = query.to_sql().unwrap();
        assert_eq!(sql, "SELECT * FROM users CROSS JOIN categories");
    }

    #[test]
    fn test_full_outer_join() {
        let query = from("users")
            .select("*")
            .full_outer_join("profiles", "users.id", "profiles.user_id");
        
        let sql = query.to_sql().unwrap();
        assert_eq!(sql, "SELECT * FROM users FULL OUTER JOIN profiles ON users.id = profiles.user_id");
    }

    #[test]
    fn test_aggregation_with_group_by() {
        let query = from("orders")
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
        let query = from("users")
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
        let query = from("orders")
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
            .order_by_asc("customer_id")
            .order_by_desc("total_sales")
            .limit(100);
            
        let sql = query.to_sql().unwrap();
        let expected = "SELECT customer_id, status, COUNT(*) AS order_count, SUM(total) AS total_sales, AVG(total) AS avg_order_value, MIN(total) AS min_order, MAX(total) AS max_order FROM orders WHERE status = ? GROUP BY customer_id, status ORDER BY customer_id ASC, total_sales DESC LIMIT 100";
        assert_eq!(sql, expected);
    }

    #[test]
    fn test_complex_distinct_query() {
        let query = from("users")
            .inner_join("user_roles", "users.id", "user_roles.user_id")
            .inner_join("roles", "user_roles.role_id", "roles.id")
            .select(("users.department", "roles.name"))
            .distinct()
            .where_(("users.active", true))
            .and_where(("roles.active", true))
            .order_by_asc("users.department")
            .order_by_asc("roles.name")
            .limit(20);
            
        let sql = query.to_sql().unwrap();
        let expected = "SELECT DISTINCT users.department, roles.name FROM users INNER JOIN user_roles ON users.id = user_roles.user_id INNER JOIN roles ON user_roles.role_id = roles.id WHERE users.active = ? AND roles.active = ? ORDER BY users.department ASC, roles.name ASC LIMIT 20";
        assert_eq!(sql, expected);
    }

    #[test]
    fn test_and_where_methods() {
        // Test that and_where works the same as where_
        let query1 = from("users")
            .select("*")
            .where_(("age", op::GT, 18))
            .where_(("status", "active"));

        let query2 = from("users")
            .select("*")
            .where_(("age", op::GT, 18))
            .and_where(("status", "active"));

        assert_eq!(query1.to_sql().unwrap(), query2.to_sql().unwrap());
    }

    #[test]
    fn test_complex_where_combinations() {
        let query = from("users")
            .select("*")
            .where_(("age", op::GTE, 18))     // First condition (AND by default)
            .and_where(("status", "active"))  // Explicit AND
            .or_where(("role", "admin"))      // OR condition
            .and_where(("verified", true));   // Back to AND

        let sql = query.to_sql().unwrap();
        assert_eq!(sql, "SELECT * FROM users WHERE age >= ? AND status = ? OR role = ? AND verified = ?");
    }

    #[test]
    fn test_distinct_all_columns() {
        let query = from("users")
            .select("*")
            .distinct();

        let sql = query.to_sql().unwrap();
        assert_eq!(sql, "SELECT DISTINCT * FROM users");
    }

    #[test]
    fn test_complex_query_with_joins_group_order() {
        let query = from("users")
            .select(("users.name", "orders.status"))
            .inner_join("orders", "users.id", "orders.user_id")
            .where_(("users.active", true))
            .group_by(("users.name", "orders.status"))
            .order_by_asc("users.name")
            .order_by_desc("orders.status")
            .limit(10);

        let sql = query.to_sql().unwrap();
        assert_eq!(sql, "SELECT users.name, orders.status FROM users INNER JOIN orders ON users.id = orders.user_id WHERE users.active = ? GROUP BY users.name, orders.status ORDER BY users.name ASC, orders.status DESC LIMIT 10");
    }

    #[test]
    fn test_distinct_multiple_columns() {
        let query = from("users")
            .select(("status", "role"))
            .distinct();

        let sql = query.to_sql().unwrap();
        assert_eq!(sql, "SELECT DISTINCT status, role FROM users");
    }

    #[test]
    fn test_distinct_with_group_by() {
        let query = from("orders")
            .group_by("customer_id")
            .select("customer_id")
            .distinct();

        let sql = query.to_sql().unwrap();
        assert_eq!(sql, "SELECT DISTINCT customer_id FROM orders GROUP BY customer_id");
    }

    #[test]
    fn test_distinct_with_join() {
        let query = from("users")
            .select("users.role")
            .distinct()
            .inner_join("departments", "users.dept_id", "departments.id");

        let sql = query.to_sql().unwrap();
        assert_eq!(sql, "SELECT DISTINCT users.role FROM users INNER JOIN departments ON users.dept_id = departments.id");
    }

    #[test]
    fn test_distinct_with_limit() {
        let query = from("users")
            .select("department")
            .distinct()
            .limit(5);

        let sql = query.to_sql().unwrap();
        assert_eq!(sql, "SELECT DISTINCT department FROM users LIMIT 5");
    }

    #[test]
    fn test_distinct_with_order_by() {
        let query = from("users")
            .select("status")
            .distinct()
            .order_by_asc("status");

        let sql = query.to_sql().unwrap();
        assert_eq!(sql, "SELECT DISTINCT status FROM users ORDER BY status ASC");
    }

    #[test]
    fn test_distinct_with_where() {
        let query = from("users")
            .where_(("active", true))
            .select("department")
            .distinct();

        let sql = query.to_sql().unwrap();
        assert_eq!(sql, "SELECT DISTINCT department FROM users WHERE active = ?");
    }


    #[test]
    fn test_group_by_with_order_by() {
        let query = from("orders")
            .select("status")
            .group_by("status")
            .order_by_asc("status");

        let sql = query.to_sql().unwrap();
        assert_eq!(sql, "SELECT status FROM orders GROUP BY status ORDER BY status ASC");
    }

    #[test]
    fn test_group_by_with_where() {
        let query = from("orders")
            .select("status")
            .where_(("active", true))
            .group_by("status");

        let sql = query.to_sql().unwrap();
        assert_eq!(sql, "SELECT status FROM orders WHERE active = ? GROUP BY status");
    }

    #[test]
    fn test_having_count_distinct() {
        let query = from("orders")
            .select(vec![
                ColumnSelector::Column("region".to_string()),
                ColumnSelector::count_distinct("customer_id").as_alias("unique_customers"),
                ColumnSelector::sum("total").as_alias("total_sales")
            ])
            .group_by("region")
            .having(("COUNT(DISTINCT customer_id)", op::GT, 100));
            
        let sql = query.to_sql().unwrap();
        assert_eq!(sql, "SELECT region, COUNT(DISTINCT(customer_id)) AS unique_customers, SUM(total) AS total_sales FROM orders GROUP BY region HAVING COUNT(DISTINCT customer_id) > ?");
    }

    #[test]
    fn test_having_with_avg() {
        let query = from("products")
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
    fn test_having_with_joins() {
        let query = from("users")
            .select(vec![
                ColumnSelector::Column("users.department".to_string()),
                ColumnSelector::count().as_alias("user_count"),
                ColumnSelector::avg("salaries.amount").as_alias("avg_salary")
            ])
            .inner_join("salaries", "users.id", "salaries.user_id")
            .group_by("users.department")
            .having(("COUNT(*)", op::GTE, 5));
            
        let sql = query.to_sql().unwrap();
        assert_eq!(sql, "SELECT users.department, COUNT(*) AS user_count, AVG(salaries.amount) AS avg_salary FROM users INNER JOIN salaries ON users.id = salaries.user_id GROUP BY users.department HAVING COUNT(*) >= ?");
    }

    #[test]
    fn test_having_with_or_condition() {
        let query = from("products")
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
    fn test_having_with_order_by() {
        let query = from("products")
            .select(vec![
                ColumnSelector::Column("category".to_string()),
                ColumnSelector::count().as_alias("product_count"),
                ColumnSelector::max("price").as_alias("max_price")
            ])
            .group_by("category")
            .having(("COUNT(*)", op::GT, 5))
            .order_by_asc("product_count")
            .order_by_desc("max_price");
            
        let sql = query.to_sql().unwrap();
        assert_eq!(sql, "SELECT category, COUNT(*) AS product_count, MAX(price) AS max_price FROM products GROUP BY category HAVING COUNT(*) > ? ORDER BY product_count ASC, max_price DESC");
    }

    #[test]
    fn test_having_with_sum() {
        let query = from("sales")
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
    fn test_join_with_limit_offset() {
        let query = from("users")
            .inner_join("profiles", "users.id", "profiles.user_id")
            .select("*")
            .limit(10)
            .offset(20);

        let sql = query.to_sql().unwrap();
        assert_eq!(sql, "SELECT * FROM users INNER JOIN profiles ON users.id = profiles.user_id LIMIT 10 OFFSET 20");
    }

    #[test]
    fn test_join_with_where_clause() {
        let query = from("users")
            .select(("users.name", "orders.total"))
            .inner_join("orders", "users.id", "orders.user_id")
            .where_(("users.active", true))
            .and_where(("orders.status", "completed"));

        let sql = query.to_sql().unwrap();
        assert_eq!(sql, "SELECT users.name, orders.total FROM users INNER JOIN orders ON users.id = orders.user_id WHERE users.active = ? AND orders.status = ?");
    }

    #[test]
    fn test_multiple_order_by() {
        let query = from("users")
            .order_by_asc("name")
            .order_by_desc("created_at")
            .order_by_asc("id")
            .select("*");

        let sql = query.to_sql().unwrap();
        assert_eq!(sql, "SELECT * FROM users ORDER BY name ASC, created_at DESC, id ASC");
    }

    #[test]
    fn test_order_by_with_limit_offset() {
        let query = from("users")
            .order_by_asc("created_at")
            .limit(25)
            .offset(50)
            .select("*");

        let sql = query.to_sql().unwrap();
        assert_eq!(sql, "SELECT * FROM users ORDER BY created_at ASC LIMIT 25 OFFSET 50");
    }

    #[test]
    fn test_order_by_with_where() {
        let query = from("users")
            .where_(("active", true))
            .order_by_asc("name")
            .select("*");

        let sql = query.to_sql().unwrap();
        assert_eq!(sql, "SELECT * FROM users WHERE active = ? ORDER BY name ASC");
    }

    #[test]
    fn test_multiple_joins() {
        let query = from("users")
            .inner_join("profiles", "users.id", "profiles.user_id")
            .left_join("orders", "users.id", "orders.user_id")
            .right_join("categories", "orders.category_id", "categories.id")
            .select("*");

        let sql = query.to_sql().unwrap();
        assert_eq!(sql, "SELECT * FROM users INNER JOIN profiles ON users.id = profiles.user_id LEFT JOIN orders ON users.id = orders.user_id RIGHT JOIN categories ON orders.category_id = categories.id");
    }

    #[test]
    fn test_mixed_columns_and_aggregations() {
        let query = from("orders")
            .select(vec![
                ColumnSelector::Column("status".to_string()),
                ColumnSelector::count().as_alias("count"),
                ColumnSelector::sum("total").as_alias("total_sales")
            ]);

        let sql = query.to_sql().unwrap();
        assert_eq!(sql, "SELECT status, COUNT(*) AS count, SUM(total) AS total_sales FROM orders");
    }

    #[test]
    fn test_having_with_where_and_group_by() {
        let query = from("orders")
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
    fn test_multiple_having_conditions() {
        let query = from("orders")
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
    fn test_generic_join_method() {
        let query = from("users")
            .join(JoinType::Inner, "profiles", "users.id", crate::Operator::EQ, "profiles.user_id")
            .select("*");

        let sql = query.to_sql().unwrap();
        assert_eq!(sql, "SELECT * FROM users INNER JOIN profiles ON users.id = profiles.user_id");
    }

    #[test]
    fn test_join_with_custom_operator() {
        let query = from("users")
            .join(JoinType::Inner, "profiles", "users.id", op::GT, "profiles.min_user_id")
            .select("*");

        let sql = query.to_sql().unwrap();
        assert_eq!(sql, "SELECT * FROM users INNER JOIN profiles ON users.id > profiles.min_user_id");
    }

    #[test]
    fn test_complex_subquery_with_joins() {
        let subquery = from("orders")
            .inner_join("order_items", "orders.id", "order_items.order_id")
            .select(ColumnSelector::sum("order_items.quantity"))
            .where_(("orders.customer_id", 1))
            .group_by("orders.customer_id");

        let query = from("customers")
            .select(vec![
                ColumnSelector::Column("name".to_string()),
                ColumnSelector::subquery_as(subquery, "total_items_ordered")
            ]);

        let sql = query.to_sql().unwrap();
        assert_eq!(sql, "SELECT name, (SELECT SUM(order_items.quantity) FROM orders INNER JOIN order_items ON orders.id = order_items.order_id WHERE orders.customer_id = ? GROUP BY orders.customer_id) AS total_items_ordered FROM customers");
    }

    #[test]
    fn test_where_in_subquery() {
        let subquery = from("orders")
            .select("customer_id")
            .where_(("status", "completed"));

        let query = from("customers")
            .where_in("id", subquery)
            .select("*");

        let sql = query.to_sql().unwrap();
        assert_eq!(sql, "SELECT * FROM customers WHERE id IN (SELECT customer_id FROM orders WHERE status = ?)");
    }

    #[test]
    fn test_subquery_in_select() {
        let subquery = from("orders")
            .select("total")
            .where_(("customer_id", 1))
            .limit(1);

        let query = from("customers")
            .select(vec![
                ColumnSelector::Column("name".to_string()),
                ColumnSelector::subquery_as(subquery, "latest_order_total")
            ]);

        let sql = query.to_sql().unwrap();
        assert_eq!(sql, "SELECT name, (SELECT total FROM orders WHERE customer_id = ? LIMIT 1) AS latest_order_total FROM customers");
    }

    #[test]
    fn test_where_exists_subquery() {
        let subquery = from("orders")
            .select("1")
            .where_(("orders.customer_id", 1));

        let query = from("customers")
            .where_exists(subquery)
            .select("*");

        let sql = query.to_sql().unwrap();
        assert_eq!(sql, "SELECT * FROM customers WHERE EXISTS (SELECT 1 FROM orders WHERE orders.customer_id = ?)");
    }

    #[test]
    fn test_where_not_in_subquery() {
        let subquery = from("cancelled_orders")
            .select("customer_id");

        let query = from("customers")
            .where_not_in("id", subquery)
            .select("*");

        let sql = query.to_sql().unwrap();
        assert_eq!(sql, "SELECT * FROM customers WHERE id NOT IN (SELECT customer_id FROM cancelled_orders)");
    }

    #[test]
    fn test_where_not_exists_subquery() {
        let subquery = from("orders")
            .select("1")
            .where_(("orders.customer_id", 1));

        let query = from("customers")
            .where_not_exists(subquery)
            .select("*");

        let sql = query.to_sql().unwrap();
        assert_eq!(sql, "SELECT * FROM customers WHERE NOT EXISTS (SELECT 1 FROM orders WHERE orders.customer_id = ?)");
    }

    #[test]
    fn test_subquery_with_aggregation() {
        let avg_subquery = from("orders")
            .select(ColumnSelector::avg("total").as_alias("avg_total"));

        let query = from("customers")
            .select(vec![
                ColumnSelector::Column("name".to_string()),
                ColumnSelector::subquery_as(avg_subquery, "avg_order_total")
            ]);

        let sql = query.to_sql().unwrap();
        assert_eq!(sql, "SELECT name, (SELECT AVG(total) AS avg_total FROM orders) AS avg_order_total FROM customers");
    }

    #[test]
    fn test_subquery_with_multiple_conditions() {
        let subquery = from("orders")
            .select("customer_id")
            .where_(("status", "completed"))
            .and_where(("total", op::GT, 50));

        let query = from("customers")
            .select("name")
            .where_in("id", subquery)
            .where_(("active", true));

        let sql = query.to_sql().unwrap();
        assert_eq!(sql, "SELECT name FROM customers WHERE active = ? AND id IN (SELECT customer_id FROM orders WHERE status = ? AND total > ?)");
    }

    #[test]
    fn test_nested_subqueries() {
        let inner_subquery = from("order_items")
            .select("order_id")
            .where_(("product_id", 1));

        let outer_subquery = from("orders")
            .select("customer_id")
            .where_in("id", inner_subquery);

        let query = from("customers")
            .select("*")
            .where_in("id", outer_subquery);

        let sql = query.to_sql().unwrap();
        assert_eq!(sql, "SELECT * FROM customers WHERE id IN (SELECT customer_id FROM orders WHERE id IN (SELECT order_id FROM order_items WHERE product_id = ?))");
    }

    #[test]
    fn test_mixed_tuple_column_selectors() {
        // Test all our new mixed tuple implementations

        // (&str, ColumnSelector)
        let query1 = from("users")
            .select(("name", ColumnSelector::count()));
        let sql1 = query1.to_sql().unwrap();
        assert_eq!(sql1, "SELECT name, COUNT(*) FROM users");

        // (&str, ColumnSelector, ColumnSelector) - the main one we wanted!
        let query2 = from("users")
            .select((
                "name",
                ColumnSelector::count().as_alias("total"),
                ColumnSelector::avg("rating").as_alias("avg_rating")
            ));
        let sql2 = query2.to_sql().unwrap();
        assert_eq!(sql2, "SELECT name, COUNT(*) AS total, AVG(rating) AS avg_rating FROM users");

        // (ColumnSelector, &str, ColumnSelector)
        let query3 = from("products")
            .select((
                ColumnSelector::sum("price").as_alias("total_price"),
                "category",
                ColumnSelector::count()
            ));
        let sql3 = query3.to_sql().unwrap();
        assert_eq!(sql3, "SELECT SUM(price) AS total_price, category, COUNT(*) FROM products");
    }

    #[test]
    fn test_mixed_where_and_subquery_conditions() {
        let subquery = from("orders")
            .select("customer_id")
            .where_(("total", op::GT, 100));

        let query = from("customers")
            .select("*")
            .where_(("active", true))
            .where_in("id", subquery);

        let sql = query.to_sql().unwrap();
        assert_eq!(sql, "SELECT * FROM customers WHERE active = ? AND id IN (SELECT customer_id FROM orders WHERE total > ?)");
    }
}
