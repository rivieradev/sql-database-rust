// Table implementation
// A table combines schema, data (pages), and indexes

use super::{btree::BTreeIndex, page::PageManager, Row, Schema, Value};
use anyhow::{anyhow, Result};
use std::collections::HashMap;

/// Represents a database table
/// This is the main structure that holds all table data
pub struct Table {
    /// The name of the table
    pub name: String,
    /// The schema (column definitions)
    pub schema: Schema,
    /// Page-based storage for rows
    page_manager: PageManager,
    /// Indexes for fast lookups
    /// HashMap is Rust's hash table - O(1) average lookup time
    indexes: HashMap<String, BTreeIndex>,
    /// The next row ID to assign
    next_row_id: usize,
}

impl Table {
    /// Create a new table with the given name and schema
    pub fn new(name: String, schema: Schema) -> Self {
        let mut table = Self {
            name,
            schema,
            page_manager: PageManager::new(100), // 100 rows per page
            indexes: HashMap::new(),
            next_row_id: 0,
        };

        // Automatically create an index on the primary key column
        if let Some(pk_index) = table.schema.get_primary_key_index() {
            let pk_name = table.schema.columns[pk_index].name.clone();
            table.create_index(&pk_name);
        }

        table
    }

    /// Insert a row into the table
    /// Returns the row ID of the inserted row
    pub fn insert(&mut self, values: Vec<Value>) -> Result<usize> {
        // Validate the row matches the schema
        if values.len() != self.schema.columns.len() {
            return Err(anyhow!(
                "Expected {} values, got {}",
                self.schema.columns.len(),
                values.len()
            ));
        }

        // Check primary key constraint (no duplicates)
        if let Some(pk_index) = self.schema.get_primary_key_index() {
            let pk_value = &values[pk_index];
            let pk_name = &self.schema.columns[pk_index].name;

            if let Some(index) = self.indexes.get(pk_name) {
                if index.lookup(pk_value).is_some() {
                    return Err(anyhow!("Primary key violation: duplicate value"));
                }
            }
        }

        // Create the row
        let row = Row { values };

        // Insert into page manager
        let (_page_id, _row_index) = self.page_manager.insert(row.clone());
        let row_id = self.next_row_id;
        self.next_row_id += 1;

        // Update all indexes
        for (col_index, value) in row.values.iter().enumerate() {
            let col_name = &self.schema.columns[col_index].name;
            if let Some(index) = self.indexes.get_mut(col_name) {
                index.insert(value.clone(), row_id);
            }
        }

        Ok(row_id)
    }

    /// Select rows based on a simple condition
    /// This is a simplified version - real databases have complex query planners
    ///
    /// Parameters:
    /// - column_name: The column to filter on (None for all rows)
    /// - value: The value to match (None for all rows)
    pub fn select(&self, column_name: Option<&str>, value: Option<&Value>) -> Result<Vec<Row>> {
        match (column_name, value) {
            // If we have a column and value, try to use an index
            (Some(col_name), Some(val)) => {
                // Check if we have an index on this column
                if let Some(index) = self.indexes.get(col_name) {
                    // Index lookup - O(log n)
                    if let Some(row_ids) = index.lookup(val) {
                        let mut results = Vec::new();
                        for &row_id in row_ids {
                            if let Some(row) = self.page_manager.get(row_id) {
                                results.push(row.clone());
                            }
                        }
                        return Ok(results);
                    } else {
                        return Ok(Vec::new());
                    }
                }

                // No index - do a full table scan
                let col_index = self
                    .schema
                    .get_column_index(col_name)
                    .ok_or_else(|| anyhow!("Column not found: {}", col_name))?;

                Ok(self
                    .page_manager
                    .scan()
                    .into_iter()
                    .filter(|(_id, row)| &row.values[col_index] == val)
                    .map(|(_id, row)| row.clone())
                    .collect())
            }
            // No filter - return all rows (full table scan)
            _ => Ok(self
                .page_manager
                .scan()
                .into_iter()
                .map(|(_id, row)| row.clone())
                .collect()),
        }
    }

    /// Update rows matching a condition
    /// Returns the number of rows updated
    pub fn update(
        &mut self,
        where_column: &str,
        where_value: &Value,
        update_column: &str,
        update_value: Value,
    ) -> Result<usize> {
        let where_col_index = self
            .schema
            .get_column_index(where_column)
            .ok_or_else(|| anyhow!("Column not found: {}", where_column))?;

        let update_col_index = self
            .schema
            .get_column_index(update_column)
            .ok_or_else(|| anyhow!("Column not found: {}", update_column))?;

        let mut updated_count = 0;

        // Find rows to update using index if available
        let row_ids: Vec<usize> = if let Some(index) = self.indexes.get(where_column) {
            index
                .lookup(where_value)
                .map(|ids| ids.clone())
                .unwrap_or_default()
        } else {
            // Full table scan
            self.page_manager
                .scan()
                .into_iter()
                .filter(|(_id, row)| &row.values[where_col_index] == where_value)
                .map(|(id, _row)| id)
                .collect()
        };

        // Update each row
        for row_id in row_ids {
            if let Some(row) = self.page_manager.get_mut(row_id) {
                // Remove old value from indexes
                let old_value = row.values[update_col_index].clone();
                if let Some(index) = self.indexes.get_mut(update_column) {
                    index.remove(&old_value, row_id);
                }

                // Update the value
                row.values[update_col_index] = update_value.clone();

                // Add new value to indexes
                if let Some(index) = self.indexes.get_mut(update_column) {
                    index.insert(update_value.clone(), row_id);
                }

                updated_count += 1;
            }
        }

        Ok(updated_count)
    }

    /// Delete rows matching a condition
    /// Note: This is simplified - real databases don't actually delete immediately
    /// They mark rows as deleted and clean up later (MVCC - Multi-Version Concurrency Control)
    pub fn delete(&mut self, column_name: &str, value: &Value) -> Result<usize> {
        let col_index = self
            .schema
            .get_column_index(column_name)
            .ok_or_else(|| anyhow!("Column not found: {}", column_name))?;

        // Find rows to delete
        let row_ids: Vec<usize> = if let Some(index) = self.indexes.get(column_name) {
            index
                .lookup(value)
                .map(|ids| ids.clone())
                .unwrap_or_default()
        } else {
            self.page_manager
                .scan()
                .into_iter()
                .filter(|(_id, row)| &row.values[col_index] == value)
                .map(|(id, _row)| id)
                .collect()
        };

        let delete_count = row_ids.len();

        // Remove from indexes
        for row_id in &row_ids {
            if let Some(row) = self.page_manager.get(*row_id) {
                for (col_idx, val) in row.values.iter().enumerate() {
                    let col_name = &self.schema.columns[col_idx].name;
                    if let Some(index) = self.indexes.get_mut(col_name) {
                        index.remove(val, *row_id);
                    }
                }
            }
        }

        Ok(delete_count)
    }

    /// Create an index on a column
    /// Indexes speed up queries but slow down inserts/updates
    pub fn create_index(&mut self, column_name: &str) -> Result<()> {
        // Check if column exists
        let col_index = self
            .schema
            .get_column_index(column_name)
            .ok_or_else(|| anyhow!("Column not found: {}", column_name))?;

        // Check if index already exists
        if self.indexes.contains_key(column_name) {
            return Err(anyhow!("Index already exists on column: {}", column_name));
        }

        // Create the index
        let mut index = BTreeIndex::new(column_name.to_string());

        // Index all existing rows
        for (row_id, row) in self.page_manager.scan() {
            let value = &row.values[col_index];
            index.insert(value.clone(), row_id);
        }

        self.indexes.insert(column_name.to_string(), index);
        Ok(())
    }

    /// Get the number of rows in the table
    pub fn row_count(&self) -> usize {
        self.page_manager.total_rows()
    }

    /// Get the schema of the table
    pub fn get_schema(&self) -> &Schema {
        &self.schema
    }
}
