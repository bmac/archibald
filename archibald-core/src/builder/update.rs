//! UPDATE query builder module

use super::common::{IntoCondition, QueryBuilder, WhereCondition, WhereConnector};
use crate::{Result, Value};

/// Initial UPDATE query builder - requires SET clause
#[derive(Debug, Clone)]
pub struct UpdateBuilderInitial {
    table_name: String,
}

/// UPDATE query builder with SET clause - requires WHERE clause
#[derive(Debug, Clone)]
pub struct UpdateBuilderWithSet {
    table_name: String,
    set_clauses: Vec<(String, Value)>,
    set_parameters: Vec<Value>,
}

/// Complete UPDATE query builder - has both SET and WHERE clauses
#[derive(Debug, Clone)]
pub struct UpdateBuilderComplete {
    table_name: String,
    set_clauses: Vec<(String, Value)>,
    where_conditions: Vec<WhereCondition>,
    where_parameters: Vec<Value>,
    all_parameters: Vec<Value>,
}

impl UpdateBuilderInitial {
    /// Create a new UPDATE query builder
    pub fn new(table: &str) -> Self {
        Self {
            table_name: table.to_string(),
        }
    }

    /// Set column values, transitioning to UpdateBuilderWithSet
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
    pub fn set<T>(self, data: T) -> UpdateBuilderWithSet
    where
        T: IntoUpdateData,
    {
        let updates = data.into_update_data();
        let set_parameters: Vec<Value> = updates.iter().map(|(_, v)| v.clone()).collect();

        UpdateBuilderWithSet {
            table_name: self.table_name,
            set_clauses: updates,
            set_parameters,
        }
    }
}

impl UpdateBuilderWithSet {
    /// Add a WHERE condition, transitioning to UpdateBuilderComplete
    pub fn where_<C>(self, condition: C) -> UpdateBuilderComplete
    where
        C: IntoCondition,
    {
        let (column, operator, value) = condition.into_condition();

        let where_condition = WhereCondition {
            column,
            operator,
            value: value.clone(),
            connector: WhereConnector::And,
        };

        let mut all_parameters = self.set_parameters.clone();
        all_parameters.push(value.clone());

        UpdateBuilderComplete {
            table_name: self.table_name,
            set_clauses: self.set_clauses,
            where_conditions: vec![where_condition],
            where_parameters: vec![value],
            all_parameters,
        }
    }

    /// Add an AND WHERE condition (same as where_)
    pub fn and_where<C>(self, condition: C) -> UpdateBuilderComplete
    where
        C: IntoCondition,
    {
        self.where_(condition)
    }
}

impl UpdateBuilderComplete {
    /// Add an additional WHERE condition with AND
    pub fn and_where<C>(mut self, condition: C) -> Self
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
        self.where_parameters.push(value.clone());
        self.all_parameters.push(value);

        self
    }

    /// Add an additional WHERE condition with OR  
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
        self.where_parameters.push(value.clone());
        self.all_parameters.push(value);

        self
    }

    /// Add a WHERE condition (same as and_where)
    pub fn where_<C>(self, condition: C) -> Self
    where
        C: IntoCondition,
    {
        self.and_where(condition)
    }
}

impl QueryBuilder for UpdateBuilderInitial {
    fn to_sql(&self) -> Result<String> {
        Err(crate::Error::invalid_query(
            "UPDATE requires SET clause. Use .set() method.",
        ))
    }

    fn parameters(&self) -> &[Value] {
        &[]
    }

    fn clone_builder(&self) -> Self {
        self.clone()
    }
}

impl QueryBuilder for UpdateBuilderWithSet {
    fn to_sql(&self) -> Result<String> {
        Err(crate::Error::invalid_query(
            "UPDATE requires WHERE clause for safety. Use .where_() or .and_where() method.",
        ))
    }

    fn parameters(&self) -> &[Value] {
        &self.set_parameters
    }

    fn clone_builder(&self) -> Self {
        self.clone()
    }
}

impl QueryBuilder for UpdateBuilderComplete {
    fn to_sql(&self) -> Result<String> {
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
        let set_parts: Vec<String> = self
            .set_clauses
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
        &self.all_parameters
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::update;
    use std::collections::HashMap;

    #[test]
    fn test_update_builder() {
        let mut data = HashMap::new();
        data.insert("name".to_string(), "John Updated".into());
        data.insert("age".to_string(), 31.into());

        let query = update("users").set(data).where_(("id", 1));

        let sql = query.to_sql().unwrap();
        // Note: HashMap iteration order is not guaranteed
        assert!(sql.starts_with("UPDATE users SET"));
        assert!(sql.contains("WHERE id = ?"));
        assert!(sql.contains("name = ?") && sql.contains("age = ?"));
    }

    #[test]
    fn test_update_without_set_fails() {
        let query = update("users");
        let result = query.to_sql();
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("UPDATE requires SET clause"));
    }

    #[test]
    fn test_update_with_set_but_no_where_fails() {
        let mut data = HashMap::new();
        data.insert("name".to_string(), "Jane".into());
        
        let query = update("users").set(data);
        let result = query.to_sql();
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("UPDATE requires WHERE clause"));
    }

    #[test]
    fn test_update_multiple_where_conditions() {
        let mut data = HashMap::new();
        data.insert("name".to_string(), "Jane".into());

        let query = update("users")
            .set(data)
            .where_(("id", 1))
            .and_where(("active", true))
            .or_where(("admin", true));

        let sql = query.to_sql().unwrap();
        assert!(sql.contains("WHERE id = ? AND active = ? OR admin = ?"));
    }

    #[test]
    fn test_type_safety_prevents_early_execution() {
        use crate::builder::common::QueryBuilder;

        // Can't call to_sql() on UpdateBuilderInitial - missing SET
        let initial_builder = update("users");
        let result = initial_builder.to_sql();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("SET clause"));

        // Can't call to_sql() on UpdateBuilderWithSet - missing WHERE  
        let mut data = HashMap::new();
        data.insert("name".to_string(), "Jane".into());
        let set_builder = update("users").set(data);
        let result = set_builder.to_sql();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("WHERE clause"));

        // CAN call to_sql() on UpdateBuilderComplete - has both SET and WHERE
        let mut data = HashMap::new();
        data.insert("name".to_string(), "Jane".into());
        let complete_builder = update("users").set(data).where_(("id", 1));
        let result = complete_builder.to_sql();
        assert!(result.is_ok());
    }
}
