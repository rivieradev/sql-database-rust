// Sharding module
// Sharding is a database scaling technique where data is split across multiple databases
// This is called "horizontal partitioning" - each shard contains a subset of rows
//
// Why sharding?
// 1. Scalability: Distribute data across multiple machines
// 2. Performance: Parallel query execution across shards
// 3. Availability: If one shard fails, others still work
//
// Common sharding strategies:
// 1. Hash-based: hash(key) % num_shards (what we implement)
// 2. Range-based: shard based on value ranges (e.g., A-M on shard1, N-Z on shard2)
// 3. Geographic: shard by location (e.g., US users on shard1, EU users on shard2)

use crate::query::{executor::QueryResult, parser::Query, QueryExecutor, QueryParser};
use crate::storage::Value;
use anyhow::Result;
use seahash::hash;

/// A sharded database that distributes data across multiple query executors
/// Each shard is an independent database instance
pub struct ShardedDatabase {
    /// The individual shards (database instances)
    shards: Vec<QueryExecutor>,
    /// Number of shards
    num_shards: usize,
}

impl ShardedDatabase {
    /// Create a new sharded database with the specified number of shards
    ///
    /// In a real distributed system, each shard would be on a different machine
    /// Here, they're all in memory for educational purposes
    pub fn new(num_shards: usize) -> Self {
        if num_shards == 0 {
            panic!("Must have at least one shard");
        }

        let mut shards = Vec::new();
        for _ in 0..num_shards {
            shards.push(QueryExecutor::new());
        }

        Self { shards, num_shards }
    }

    /// Execute a SQL query against the sharded database
    pub fn execute(&mut self, sql: &str) -> Result<QueryResult> {
        // Parse the SQL query
        let query = QueryParser::parse(sql)?;

        match &query {
            // For CREATE TABLE, we need to create the table on ALL shards
            // This ensures every shard has the same schema
            Query::CreateTable { .. } => {
                for shard in &mut self.shards {
                    shard.execute(QueryParser::parse(sql)?)?;
                }
                Ok(QueryResult::Message("Table created on all shards".to_string()))
            }

            // For CREATE INDEX, apply to all shards
            Query::CreateIndex { .. } => {
                for shard in &mut self.shards {
                    shard.execute(QueryParser::parse(sql)?)?;
                }
                Ok(QueryResult::Message("Index created on all shards".to_string()))
            }

            // For INSERT, we route to a specific shard based on the primary key
            Query::Insert { values, .. } => {
                // Use the first value (usually the primary key) for sharding
                // In a real system, you'd explicitly specify the shard key
                let shard_id = self.get_shard_id(&values[0]);
                self.shards[shard_id].execute(query)
            }

            // For SELECT with WHERE clause, we can route to a specific shard
            Query::Select {
                where_clause: Some(where_clause),
                ..
            } => {
                let shard_id = self.get_shard_id(&where_clause.value);
                self.shards[shard_id].execute(query)
            }

            // For SELECT without WHERE, we need to query ALL shards and merge results
            // This is called a "scatter-gather" query
            Query::Select {
                where_clause: None,
                table_name: _,
            } => {
                let mut all_rows = Vec::new();
                let mut column_names = Vec::new();

                // Query each shard
                for shard in &mut self.shards {
                    let result = shard.execute(QueryParser::parse(sql)?)?;
                    match result {
                        QueryResult::Rows { rows, column_names: cols } => {
                            if column_names.is_empty() {
                                column_names = cols;
                            }
                            all_rows.extend(rows);
                        }
                        _ => {}
                    }
                }

                Ok(QueryResult::Rows {
                    rows: all_rows,
                    column_names,
                })
            }

            // For UPDATE/DELETE with WHERE, route to specific shard
            Query::Update { where_clause, .. } | Query::Delete { where_clause, .. } => {
                let shard_id = self.get_shard_id(&where_clause.value);
                self.shards[shard_id].execute(query)
            }
        }
    }

    /// Determine which shard a value belongs to
    /// This uses consistent hashing to distribute data evenly
    ///
    /// The hash function takes any value and produces a number
    /// We then use modulo (%) to map it to a shard
    fn get_shard_id(&self, value: &Value) -> usize {
        // Convert the value to bytes for hashing
        let bytes = match value {
            Value::Integer(i) => i.to_string().into_bytes(),
            Value::Float(f) => f.to_string().into_bytes(),
            Value::Text(s) => s.as_bytes().to_vec(),
            Value::Boolean(b) => b.to_string().into_bytes(),
            Value::Null => b"null".to_vec(),
        };

        // Hash the bytes using SeaHash (a fast, high-quality hash function)
        let hash_value = hash(&bytes);

        // Map to a shard using modulo
        // This ensures even distribution across shards
        (hash_value as usize) % self.num_shards
    }

    /// Get the number of shards
    pub fn shard_count(&self) -> usize {
        self.num_shards
    }

    /// Get statistics about data distribution across shards
    /// This is useful for monitoring shard balance
    pub fn get_shard_stats(&self, table_name: &str) -> Vec<ShardStats> {
        let mut stats = Vec::new();

        for (i, shard) in self.shards.iter().enumerate() {
            let row_count = shard
                .get_table(table_name)
                .map(|t| t.row_count())
                .unwrap_or(0);

            stats.push(ShardStats {
                shard_id: i,
                row_count,
            });
        }

        stats
    }
}

/// Statistics for a single shard
#[derive(Debug)]
pub struct ShardStats {
    pub shard_id: usize,
    pub row_count: usize,
}

impl ShardStats {
    pub fn format(&self) -> String {
        format!("Shard {}: {} rows", self.shard_id, self.row_count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sharding_distribution() {
        let mut db = ShardedDatabase::new(3);

        // Create a table
        db.execute("CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT)")
            .unwrap();

        // Insert some data
        for i in 1..=10 {
            db.execute(&format!(
                "INSERT INTO users VALUES ({}, 'User{}')",
                i, i
            ))
            .unwrap();
        }

        // Check shard distribution
        let stats = db.get_shard_stats("users");
        let total: usize = stats.iter().map(|s| s.row_count).sum();
        assert_eq!(total, 10);

        // Each shard should have some data (not perfect distribution, but close)
        for stat in stats {
            println!("{}", stat.format());
        }
    }
}
