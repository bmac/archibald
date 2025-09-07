//! DELETE query builder module

use crate::{Result, Error, Value};
use super::common::{QueryBuilder, IntoCondition, WhereCondition, WhereConnector};

/// DELETE query builder in initial state (before where_() is called)
/// Can build conditions but cannot execute queries
#[derive(Debug, Clone)]
pub struct DeleteBuilderInitial {
    table_name: String,
}

/// DELETE query builder in complete state (after where_() is called)
/// Can execute queries and add more WHERE conditions
#[derive(Debug, Clone)]
pub struct DeleteBuilderComplete {
    table_name: String,
    where_conditions: Vec<WhereCondition>,
    parameters: Vec<Value>,
}

impl DeleteBuilderInitial {
    /// Create a new DELETE query builder in initial state
    pub fn new(table: &str) -> Self {
        Self {
            table_name: table.to_string(),
        }
    }

    /// Add a WHERE condition - transitions to DeleteBuilderComplete
    /// This is required before the query can be executed
    pub fn where_<C>(self, condition: C) -> DeleteBuilderComplete
    where
        C: IntoCondition,
    {
        let (column, operator, value) = condition.into_condition();

        let mut where_conditions = Vec::new();
        let mut parameters = Vec::new();

        where_conditions.push(WhereCondition {
            column,
            operator,
            value: value.clone(),
            connector: WhereConnector::And,
        });
        parameters.push(value);

        DeleteBuilderComplete {
            table_name: self.table_name,
            where_conditions,
            parameters,
        }
    }
}

impl DeleteBuilderComplete {
    /// Add a WHERE condition
    pub fn where_<C>(mut self, condition: C) -> Self
    where
        C: IntoCondition,
    {
        let (column, operator, value) = condition.into_condition();

        self.where_conditions.push(WhereCondition {
            column,
            operator,
            value: value.clone(),
            connector: WhereConnector::And,
        });
        self.parameters.push(value);

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
            value: value.clone(),
            connector: WhereConnector::Or,
        });
        self.parameters.push(value);

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

impl QueryBuilder for DeleteBuilderInitial {
    fn to_sql(&self) -> Result<String> {
        Err(Error::invalid_query("DELETE requires WHERE condition for safety"))
    }

    fn parameters(&self) -> &[Value] {
        &[]
    }

    fn clone_builder(&self) -> Self {
        self.clone()
    }
}

impl QueryBuilder for DeleteBuilderComplete {
    fn to_sql(&self) -> Result<String> {
        // Validate all operators before generating SQL
        for condition in &self.where_conditions {
            condition.operator.validate()?;
        }

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::delete;
    use crate::operator::op;

    #[test]
    fn test_delete_builder() {
        let query = delete("users").where_(("age", op::LT, 18));
        let sql = query.to_sql().unwrap();
        assert_eq!(sql, "DELETE FROM users WHERE age < ?");
    }

    #[test]
    fn test_delete_with_where_required() {
        // DELETE now requires WHERE condition for safety
        let query = delete("users").where_(("id", 1));
        let sql = query.to_sql().unwrap();
        assert_eq!(sql, "DELETE FROM users WHERE id = ?");
    }

    #[test]
    fn test_delete_multiple_conditions() {
        let query = delete("users")
            .where_(("age", op::LT, 18))
            .or_where(("status", "inactive"));
        let sql = query.to_sql().unwrap();
        assert_eq!(sql, "DELETE FROM users WHERE age < ? OR status = ?");
    }

    #[test]
    fn test_delete_without_where_fails() {
        let query = delete("users");
        let result = query.to_sql();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("DELETE requires WHERE condition for safety"));
    }
}