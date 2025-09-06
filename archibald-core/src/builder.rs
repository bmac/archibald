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

/// SELECT query builder
#[derive(Debug, Clone)]
pub struct SelectBuilder {
    table_name: String,
    selected_columns: Vec<String>,
    where_conditions: Vec<WhereCondition>,
    limit_value: Option<u64>,
    offset_value: Option<u64>,
    parameters: Vec<Value>,
}

impl SelectBuilder {
    /// Create a new SELECT query builder
    pub fn new(table: &str) -> Self {
        Self {
            table_name: table.to_string(),
            selected_columns: vec!["*".to_string()],
            where_conditions: Vec::new(),
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
        T: IntoColumns,
    {
        self.selected_columns = columns.into_columns();
        self
    }
    
    /// Select all columns (equivalent to SELECT *)
    pub fn select_all(mut self) -> Self {
        self.selected_columns = vec!["*".to_string()];
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
}

impl QueryBuilder for SelectBuilder {
    fn to_sql(&self) -> Result<String> {
        let mut sql = String::new();
        
        // SELECT clause
        sql.push_str("SELECT ");
        sql.push_str(&self.selected_columns.join(", "));
        
        // FROM clause
        sql.push_str(" FROM ");
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
}