// RustyDB - A simple SQL database implementation in Rust
// This is the library root that exposes the public API

pub mod query;
pub mod sharding;
pub mod storage;

// Re-export commonly used types for convenience
pub use query::{executor::QueryExecutor, parser::QueryParser};
pub use sharding::ShardedDatabase;
pub use storage::{Column, DataType, Row, Schema, Value};
