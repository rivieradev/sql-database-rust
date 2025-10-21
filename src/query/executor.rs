// Query Executor
// This module executes parsed queries against the database

use super::parser::{Query, WhereClause};
use crate::storage::{table::Table, Row};
use anyhow::{anyhow, Result};
use std::collections::HashMap;

/// The query executor manages all tables and executes queries
/// This is the main interface to the database
pub struct QueryExecutor {
    /// HashMap storing all tables by name
    /// The String is the table name, the Table is the table itself
    tables: HashMap<String, Table>,
}

impl QueryExecutor {
    /// Create a new query executor (empty database)
    pub fn new() -> Self {
        Self {
            tables: HashMap::new(),
        }
    }

    /// Execute a query and return the result
    /// Returns a QueryResult which can be rows, a count, or a message
    pub fn execute(&mut self, query: Query) -> Result<QueryResult> {
        match query {
            Query::CreateTable { name, schema } => {
                // Check if table already exists
                if self.tables.contains_key(&name) {
                    return Err(anyhow!("Table '{}' already exists", name));
                }

                // Create the table
                let table = Table::new(name.clone(), schema);
                self.tables.insert(name.clone(), table);

                Ok(QueryResult::Message(format!("Table '{}' created", name)))
            }

            Query::Insert { table_name, values } => {
                // Get the table (mut reference so we can modify it)
                let table = self
                    .tables
                    .get_mut(&table_name)
                    .ok_or_else(|| anyhow!("Table '{}' not found", table_name))?;

                // Insert the row
                table.insert(values)?;

                Ok(QueryResult::Message(format!(
                    "1 row inserted into '{}'",
                    table_name
                )))
            }

            Query::Select {
                table_name,
                where_clause,
            } => {
                // Get the table
                let table = self
                    .tables
                    .get(&table_name)
                    .ok_or_else(|| anyhow!("Table '{}' not found", table_name))?;

                // Execute the select
                let rows = match where_clause {
                    Some(WhereClause { column, value }) => {
                        table.select(Some(&column), Some(&value))?
                    }
                    None => table.select(None, None)?,
                };

                Ok(QueryResult::Rows {
                    rows,
                    column_names: table
                        .get_schema()
                        .columns
                        .iter()
                        .map(|c| c.name.clone())
                        .collect(),
                })
            }

            Query::Update {
                table_name,
                set_column,
                set_value,
                where_clause,
            } => {
                let table = self
                    .tables
                    .get_mut(&table_name)
                    .ok_or_else(|| anyhow!("Table '{}' not found", table_name))?;

                let count = table.update(
                    &where_clause.column,
                    &where_clause.value,
                    &set_column,
                    set_value,
                )?;

                Ok(QueryResult::Message(format!(
                    "{} row(s) updated in '{}'",
                    count, table_name
                )))
            }

            Query::Delete {
                table_name,
                where_clause,
            } => {
                let table = self
                    .tables
                    .get_mut(&table_name)
                    .ok_or_else(|| anyhow!("Table '{}' not found", table_name))?;

                let count = table.delete(&where_clause.column, &where_clause.value)?;

                Ok(QueryResult::Message(format!(
                    "{} row(s) deleted from '{}'",
                    count, table_name
                )))
            }

            Query::CreateIndex {
                table_name,
                column_name,
            } => {
                let table = self
                    .tables
                    .get_mut(&table_name)
                    .ok_or_else(|| anyhow!("Table '{}' not found", table_name))?;

                table.create_index(&column_name)?;

                Ok(QueryResult::Message(format!(
                    "Index created on '{}.{}'",
                    table_name, column_name
                )))
            }
        }
    }

    /// Get a reference to a table (useful for direct access)
    pub fn get_table(&self, name: &str) -> Option<&Table> {
        self.tables.get(name)
    }

    /// List all tables in the database
    pub fn list_tables(&self) -> Vec<String> {
        self.tables.keys().cloned().collect()
    }
}

/// Represents the result of a query execution
#[derive(Debug)]
pub enum QueryResult {
    /// Rows returned from a SELECT query
    Rows {
        rows: Vec<Row>,
        column_names: Vec<String>,
    },
    /// A message (for CREATE, INSERT, UPDATE, DELETE)
    Message(String),
}

impl QueryResult {
    /// Format the result as a string for display
    /// This creates a nice table format for SELECT results
    pub fn format(&self) -> String {
        match self {
            QueryResult::Message(msg) => msg.clone(),
            QueryResult::Rows { rows, column_names } => {
                if rows.is_empty() {
                    return "No rows found".to_string();
                }

                // Calculate column widths
                let mut widths: Vec<usize> = column_names.iter().map(|c| c.len()).collect();

                for row in rows {
                    for (i, value) in row.values.iter().enumerate() {
                        widths[i] = widths[i].max(value.to_string().len());
                    }
                }

                let mut result = String::new();

                // Header row
                result.push_str("┌");
                for (i, width) in widths.iter().enumerate() {
                    result.push_str(&"─".repeat(width + 2));
                    if i < widths.len() - 1 {
                        result.push_str("┬");
                    }
                }
                result.push_str("┐\n");

                // Column names
                result.push_str("│");
                for (name, width) in column_names.iter().zip(&widths) {
                    result.push_str(&format!(" {:<width$} ", name, width = width));
                    result.push_str("│");
                }
                result.push('\n');

                // Separator
                result.push_str("├");
                for (i, width) in widths.iter().enumerate() {
                    result.push_str(&"─".repeat(width + 2));
                    if i < widths.len() - 1 {
                        result.push_str("┼");
                    }
                }
                result.push_str("┤\n");

                // Data rows
                for row in rows {
                    result.push_str("│");
                    for (value, width) in row.values.iter().zip(&widths) {
                        result.push_str(&format!(" {:<width$} ", value.to_string(), width = width));
                        result.push_str("│");
                    }
                    result.push('\n');
                }

                // Bottom border
                result.push_str("└");
                for (i, width) in widths.iter().enumerate() {
                    result.push_str(&"─".repeat(width + 2));
                    if i < widths.len() - 1 {
                        result.push_str("┴");
                    }
                }
                result.push_str("┘\n");

                result.push_str(&format!("\n{} row(s) returned", rows.len()));

                result
            }
        }
    }
}

impl Default for QueryExecutor {
    fn default() -> Self {
        Self::new()
    }
}
