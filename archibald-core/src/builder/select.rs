//! SELECT query builder implementation

use crate::{Result, Error, Value};
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
                sql.push(' ');
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
}