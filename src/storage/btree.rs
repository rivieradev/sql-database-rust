// B-Tree Index Implementation
// B-Trees are the foundation of database indexing - they keep data sorted and allow fast lookups
// A B-Tree is a self-balancing tree where each node can have multiple children
// This makes it perfect for disk-based storage (databases)

use super::Value;
use std::collections::BTreeMap;

/// Index structure using Rust's built-in BTreeMap
/// BTreeMap is a sorted map that uses a B-Tree internally
///
/// Why B-Trees for databases?
/// 1. Sorted data: Keys are always in order
/// 2. O(log n) lookups: Very fast even with millions of rows
/// 3. Range queries: Easy to find all values between X and Y
/// 4. Disk-friendly: Minimizes disk reads by grouping data
#[derive(Debug, Clone)]
pub struct BTreeIndex {
    /// Maps index key (Value) to row IDs
    /// The row ID is a usize (unsigned integer) that identifies the row position
    tree: BTreeMap<IndexKey, Vec<usize>>,
    /// Name of the indexed column
    column_name: String,
}

/// Wrapper for Value to make it ordered (Ord trait)
/// Rust requires types in BTreeMap to be orderable
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IndexKey(pub Value);

// Implement ordering for IndexKey
// This is required for BTreeMap to sort the keys
impl PartialOrd for IndexKey {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.0.compare(&other.0)
    }
}

impl Ord for IndexKey {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // For values that can't be compared, we treat them as equal
        // This is a simplification for this educational database
        self.partial_cmp(other).unwrap_or(std::cmp::Ordering::Equal)
    }
}

impl BTreeIndex {
    /// Create a new B-Tree index for a specific column
    pub fn new(column_name: String) -> Self {
        Self {
            tree: BTreeMap::new(),
            column_name,
        }
    }

    /// Insert a value into the index
    ///
    /// Parameters:
    /// - value: The column value to index
    /// - row_id: The ID of the row containing this value
    pub fn insert(&mut self, value: Value, row_id: usize) {
        // 'mut self' means we can modify the index
        let key = IndexKey(value);

        // entry() is a powerful Rust API for HashMap/BTreeMap
        // It avoids double lookups (check if exists, then insert)
        self.tree
            .entry(key)
            .or_insert_with(Vec::new) // Create empty Vec if key doesn't exist
            .push(row_id);
    }

    /// Look up a value in the index
    /// Returns a reference to the vector of row IDs (if found)
    ///
    /// The '&' means we return a reference (borrowing), not ownership
    /// Option<T> is Rust's way of handling null - it's either Some(T) or None
    pub fn lookup(&self, value: &Value) -> Option<&Vec<usize>> {
        let key = IndexKey(value.clone());
        self.tree.get(&key)
    }

    /// Range query: find all values between min and max
    /// This demonstrates the power of B-Trees for range queries
    ///
    /// Returns: Vector of row IDs matching the range
    pub fn range_query(&self, min: &Value, max: &Value) -> Vec<usize> {
        let min_key = IndexKey(min.clone());
        let max_key = IndexKey(max.clone());

        let mut result = Vec::new();

        // range() gives us an iterator over all entries between min and max
        // This is O(log n + k) where k is the number of results
        for (_key, row_ids) in self.tree.range(min_key..=max_key) {
            result.extend(row_ids);
        }

        result
    }

    /// Remove a value from the index
    pub fn remove(&mut self, value: &Value, row_id: usize) {
        let key = IndexKey(value.clone());

        // if let is Rust's way to handle Option types
        // It runs the block only if the value is Some(...)
        if let Some(row_ids) = self.tree.get_mut(&key) {
            // Remove the row_id from the vector
            row_ids.retain(|&id| id != row_id);

            // If no more rows have this value, remove the key entirely
            if row_ids.is_empty() {
                self.tree.remove(&key);
            }
        }
    }

    /// Get the column name this index is for
    pub fn column_name(&self) -> &str {
        // Returns a string slice (reference to the String)
        &self.column_name
    }

    /// Get the number of unique values in the index
    pub fn len(&self) -> usize {
        self.tree.len()
    }

    /// Check if the index is empty
    pub fn is_empty(&self) -> bool {
        self.tree.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_btree_insert_and_lookup() {
        let mut index = BTreeIndex::new("id".to_string());

        index.insert(Value::Integer(1), 0);
        index.insert(Value::Integer(2), 1);
        index.insert(Value::Integer(1), 2); // Duplicate value, different row

        let result = index.lookup(&Value::Integer(1));
        assert_eq!(result, Some(&vec![0, 2]));
    }

    #[test]
    fn test_btree_range_query() {
        let mut index = BTreeIndex::new("age".to_string());

        index.insert(Value::Integer(25), 0);
        index.insert(Value::Integer(30), 1);
        index.insert(Value::Integer(35), 2);
        index.insert(Value::Integer(40), 3);

        let result = index.range_query(&Value::Integer(28), &Value::Integer(36));
        assert!(result.contains(&1));
        assert!(result.contains(&2));
        assert!(!result.contains(&0));
        assert!(!result.contains(&3));
    }
}
