//! INSERT query builder implementations

use super::common::QueryBuilder;
use crate::{Error, Result, Value};

/// INSERT query builder in initial state (before values() is called)
/// Can build conditions but cannot execute queries
#[derive(Debug, Clone)]
pub struct InsertBuilderInitial {
    table_name: String,
}

/// INSERT query builder in complete state (after values() is called)
/// Can execute queries but cannot call values() again
#[derive(Debug, Clone)]
pub struct InsertBuilderComplete {
    table_name: String,
    columns: Vec<String>,
    values: Vec<Vec<Value>>,
    parameters: Vec<Value>,
}

impl InsertBuilderInitial {
    /// Create a new INSERT query builder in initial state
    pub fn new(table: &str) -> Self {
        Self {
            table_name: table.to_string(),
        }
    }

    /// Add values for a single record, transitioning to InsertBuilderComplete
    ///
    /// # Examples
    /// ```
    /// use archibald::insert;
    /// use std::collections::HashMap;
    ///
    /// let mut data = HashMap::new();
    /// data.insert("name".to_string(), "John".into());
    /// data.insert("age".to_string(), 30.into());
    ///
    /// let query = insert("users").values(data);
    /// ```
    pub fn values<T>(self, data: T) -> InsertBuilderComplete
    where
        T: IntoInsertData,
    {
        let (columns, values) = data.into_insert_data();
        InsertBuilderComplete {
            table_name: self.table_name,
            columns,
            values: vec![values],
            parameters: Vec::new(),
        }
    }

    /// Add values for multiple records, transitioning to InsertBuilderComplete
    pub fn values_many<T>(self, data: Vec<T>) -> InsertBuilderComplete
    where
        T: IntoInsertData + Clone,
    {
        let mut columns = Vec::new();
        let mut values_vec = Vec::new();

        if let Some(first) = data.first() {
            let (cols, _) = first.clone().into_insert_data();
            columns = cols;

            for item in data {
                let (_, vals) = item.into_insert_data();
                values_vec.push(vals);
            }
        }

        InsertBuilderComplete {
            table_name: self.table_name,
            columns,
            values: values_vec,
            parameters: Vec::new(),
        }
    }
}

impl QueryBuilder for InsertBuilderInitial {
    fn to_sql(&self) -> Result<String> {
        Err(Error::invalid_query(
            "INSERT requires values to be specified with .values()",
        ))
    }

    fn parameters(&self) -> &[Value] {
        &[]
    }

    fn clone_builder(&self) -> Self {
        self.clone()
    }
}

impl QueryBuilder for InsertBuilderComplete {
    fn to_sql(&self) -> Result<String> {
        if self.columns.is_empty() || self.values.is_empty() {
            return Err(crate::Error::invalid_query(
                "INSERT requires columns and values",
            ));
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
        let value_groups: Vec<String> = self
            .values
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::insert;
    use std::collections::HashMap;

    #[test]
    fn test_insert_builder() {
        let mut data = HashMap::new();
        data.insert("name".to_string(), "John".into());
        data.insert("age".to_string(), 30.into());

        let query = insert("users").values(data);
        let sql = query.to_sql().unwrap();
        // Note: HashMap iteration order is not guaranteed, so we check both possible orders
        assert!(
            sql == "INSERT INTO users (name, age) VALUES (?, ?)"
                || sql == "INSERT INTO users (age, name) VALUES (?, ?)"
        );
    }

    #[test]
    fn test_insert_many() {
        let mut data1 = HashMap::new();
        data1.insert("name".to_string(), "John".into());
        data1.insert("age".to_string(), 30.into());

        let mut data2 = HashMap::new();
        data2.insert("name".to_string(), "Jane".into());
        data2.insert("age".to_string(), 25.into());

        let query = insert("users").values_many(vec![data1, data2]);
        let sql = query.to_sql().unwrap();
        // Check that we have multiple value groups
        assert!(sql.contains("VALUES (?, ?), (?, ?)"));
        assert!(sql.starts_with("INSERT INTO users"));
    }

    #[test]
    fn test_insert_empty_data_fails() {
        let query = insert("users");
        let result = query.to_sql();
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("INSERT requires values"));
    }
}
