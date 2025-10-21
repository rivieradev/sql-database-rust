// Storage module - handles data persistence and indexing
// This module contains the core storage engine for our database

pub mod btree;
pub mod page;
pub mod table;

use serde::{Deserialize, Serialize};

/// Represents a single row in a table
/// In Rust, we use Vec<Value> to represent a row where each Value is a column
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Row {
    pub values: Vec<Value>,
}

/// Represents different data types that can be stored in the database
/// This is called an "enum" in Rust - it can be one of several variants
/// The Serialize and Deserialize traits allow us to convert to/from JSON
/// Note: We derive Eq even though Float doesn't strictly support it
/// This is a simplification for educational purposes
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Value {
    Null,
    Integer(i64),
    // For Float, we'll use i64 representation to allow Eq
    // In a real database, you'd handle floats more carefully
    Float(i64), // Stored as integer representation (multiply by 1000 for precision)
    Text(String),
    Boolean(bool),
}

impl Value {
    /// Convert Value to a string representation
    /// The 'impl' keyword implements methods for a type
    pub fn to_string(&self) -> String {
        match self {
            // The 'match' keyword is Rust's pattern matching - like a powerful switch statement
            Value::Null => "NULL".to_string(),
            Value::Integer(i) => i.to_string(),
            Value::Float(f) => {
                // Convert back to float representation (divided by 1000)
                let float_val = (*f as f64) / 1000.0;
                float_val.to_string()
            }
            Value::Text(s) => s.clone(),
            Value::Boolean(b) => b.to_string(),
        }
    }

    /// Compare two values (used for WHERE clauses)
    /// The '&self' means this method borrows 'self' (doesn't take ownership)
    pub fn compare(&self, other: &Value) -> Option<std::cmp::Ordering> {
        match (self, other) {
            (Value::Integer(a), Value::Integer(b)) => Some(a.cmp(b)),
            (Value::Float(a), Value::Float(b)) => Some(a.cmp(b)),
            (Value::Text(a), Value::Text(b)) => Some(a.cmp(b)),
            (Value::Boolean(a), Value::Boolean(b)) => Some(a.cmp(b)),
            _ => None, // Can't compare different types
        }
    }
}

/// Represents the schema (structure) of a table
/// This defines what columns exist and their data types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Schema {
    pub columns: Vec<Column>,
}

/// Represents a single column definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Column {
    pub name: String,
    pub data_type: DataType,
    pub primary_key: bool,
    pub nullable: bool,
}

/// The data types our database supports
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DataType {
    Integer,
    Float,
    Text,
    Boolean,
}

impl Schema {
    /// Create a new schema with the given columns
    pub fn new(columns: Vec<Column>) -> Self {
        // 'Self' refers to the type we're implementing (Schema)
        Self { columns }
    }

    /// Find the index of a column by name
    /// Returns Option<usize> - either Some(index) or None if not found
    pub fn get_column_index(&self, name: &str) -> Option<usize> {
        self.columns.iter().position(|col| col.name == name)
    }

    /// Get the primary key column index
    pub fn get_primary_key_index(&self) -> Option<usize> {
        self.columns.iter().position(|col| col.primary_key)
    }
}
