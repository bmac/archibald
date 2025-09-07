//! UPDATE query builder module

use crate::{Result, Value};
use super::common::{QueryBuilder, IntoCondition, WhereCondition, WhereConnector};

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
    /// use archibald_core::update;
    /// use std::collections::HashMap;
    ///
    /// let mut updates = HashMap::new();
    /// updates.insert("name".to_string(), "Jane".into());
    /// updates.insert("age".to_string(), 25.into());
    ///
    /// let query = update("users").set(updates);
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
}

impl QueryBuilder for UpdateBuilder {
    fn to_sql(&self) -> Result<String> {
        if self.set_clauses.is_empty() {
            return Err(crate::Error::invalid_query("UPDATE requires SET clauses"));
        }

        // Validate all operators before generating SQL
        for condition in &self.where_conditions {
            condition.operator.validate()?;
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

/// Trait for types that can be converted to UPDATE data
pub trait IntoUpdateData {
    fn into_update_data(self) -> Vec<(String, Value)>;
}

impl IntoUpdateData for std::collections::HashMap<String, Value> {
    fn into_update_data(self) -> Vec<(String, Value)> {
        self.into_iter().collect()
    }
}