# RustyDB - A Simple SQL Database in Rust

An educational SQL database implementation in Rust, designed to teach both Rust programming and fundamental database concepts.

## 🎯 Features

- **Full SQL Support**: CREATE TABLE, INSERT, SELECT, UPDATE, DELETE
- **B-Tree Indexing**: Fast lookups with O(log n) complexity
- **Page-Based Storage**: Efficient disk-like storage simulation
- **Sharding Support**: Horizontal partitioning for scalability
- **Interactive CLI**: REPL interface for executing SQL commands
- **Well-Commented Code**: Extensive documentation explaining Rust and database concepts

## 🚀 Getting Started

### Prerequisites

- Rust 1.70+ (install from [rustup.rs](https://rustup.rs))

### Installation

```bash
# Clone or navigate to the project directory
cd SQLDatabaseRust

# Build the project
cargo build --release

# Run the database
cargo run --release
```

## 📖 Basic Commands

### Interactive Mode

Start the interactive shell:

```bash
cargo run --release
```

You'll see:
```
╔════════════════════════════════════════════╗
║         RustyDB Interactive Shell         ║
║      A Simple SQL Database in Rust        ║
╚════════════════════════════════════════════╝

Type SQL commands or '.help' for help
Type '.exit' to quit

rustydb>
```

### Single Command Mode

Execute a single SQL command:

```bash
cargo run --release -- -e "SELECT * FROM users"
```

### Sharded Mode

Run with multiple shards (for horizontal partitioning):

```bash
# Run with 4 shards
cargo run --release -- --shards 4
```

## 💡 SQL Examples

### 1. Creating a Table

```sql
CREATE TABLE users (
    id INTEGER PRIMARY KEY,
    name TEXT,
    email TEXT,
    age INTEGER,
    active BOOLEAN
)
```

**What this teaches:**
- `CREATE TABLE` defines the structure (schema) of your data
- Each column has a name and data type
- `PRIMARY KEY` ensures unique identifiers and creates an automatic index

### 2. Inserting Data

```sql
INSERT INTO users VALUES (1, 'Alice Johnson', 'alice@email.com', 30, true)
INSERT INTO users VALUES (2, 'Bob Smith', 'bob@email.com', 25, true)
INSERT INTO users VALUES (3, 'Charlie Brown', 'charlie@email.com', 35, false)
```

**What this teaches:**
- Data is inserted row by row
- Values must match the schema (column order and types)
- The database stores data in pages (not individual rows)

### 3. Querying Data

#### Select all rows:
```sql
SELECT * FROM users
```

#### Select with a filter:
```sql
SELECT * FROM users WHERE id = 2
```

**What this teaches:**
- `SELECT *` retrieves all columns
- `WHERE` clause filters results
- If an index exists on the WHERE column, the query is much faster!

### 4. Creating an Index

```sql
CREATE INDEX ON users (email)
```

**What this teaches:**
- Indexes speed up queries on specific columns
- Uses B-Tree data structure for O(log n) lookups
- Trade-off: faster queries, slower inserts

### 5. Updating Data

```sql
UPDATE users SET age = 31 WHERE id = 1
```

**What this teaches:**
- `UPDATE` modifies existing rows
- Always requires `WHERE` clause (safety feature)
- Indexes are automatically updated

### 6. Deleting Data

```sql
DELETE FROM users WHERE id = 3
```

**What this teaches:**
- `DELETE` removes rows
- Always requires `WHERE` clause (safety feature)
- Rows are removed from indexes too

## 🏗️ Architecture Overview

### Storage Layer (`src/storage/`)

#### 1. **Values and Rows** (`mod.rs`)
- Defines the data types: Integer, Float, Text, Boolean, Null
- Rows are vectors of values
- Schemas define table structure

#### 2. **B-Tree Indexes** (`btree.rs`)
- Uses Rust's `BTreeMap` for sorted key-value storage
- O(log n) lookups, inserts, and deletes
- Supports range queries efficiently
- Automatically maintained when data changes

**Why B-Trees?**
- Keeps data sorted
- Balanced tree structure (guaranteed O(log n))
- Disk-friendly (minimizes I/O operations)
- Perfect for databases!

#### 3. **Page-Based Storage** (`page.rs`)
- Data is stored in fixed-size pages (like real databases)
- Pages contain multiple rows (currently 100 rows per page)
- Simulates disk block storage

**Why Pages?**
- Disks read/write in blocks, not individual bytes
- Caching entire pages improves performance
- More efficient than row-by-row storage

#### 4. **Tables** (`table.rs`)
- Combines schema + data + indexes
- Handles INSERT, SELECT, UPDATE, DELETE
- Automatically maintains indexes
- Enforces primary key constraints

### Query Layer (`src/query/`)

#### 1. **Parser** (`parser.rs`)
- Converts SQL strings into structured queries
- Uses the `sqlparser` crate for robust SQL parsing
- Validates syntax and extracts query components

#### 2. **Executor** (`executor.rs`)
- Executes parsed queries against tables
- Manages all tables in a database
- Formats results for display

### Sharding Layer (`src/sharding/`)

Implements horizontal partitioning (sharding):

- **Hash-based sharding**: Uses hash(key) % num_shards
- **Scatter-gather**: Queries without WHERE scan all shards
- **Shard routing**: Queries with WHERE go to specific shard

**Why Sharding?**
- Distribute data across multiple machines
- Scale horizontally (add more machines)
- Parallel query execution
- Fault tolerance

## 🧠 Key Rust Concepts Demonstrated

### 1. Ownership and Borrowing
```rust
pub fn select(&self, column_name: Option<&str>) -> Result<Vec<Row>>
//            ^self is borrowed (read-only)
```
- `&self` - borrow immutably (read-only)
- `&mut self` - borrow mutably (can modify)
- Ownership prevents data races and memory leaks

### 2. Pattern Matching
```rust
match value {
    Value::Integer(i) => i.to_string(),
    Value::Text(s) => s.clone(),
    _ => "unknown".to_string(),
}
```
- Powerful alternative to if/else chains
- Compiler ensures all cases are handled

### 3. Error Handling with Result
```rust
pub fn insert(&mut self, values: Vec<Value>) -> Result<usize>
//                                              ^^^^^^^^^
```
- `Result<T, E>` represents success (Ok) or error (Err)
- Forces explicit error handling (no exceptions!)
- Use `?` operator to propagate errors

### 4. Traits
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
```
- Traits are like interfaces
- `Serialize`/`Deserialize` enable JSON conversion
- `Clone` allows copying data
- `Debug` enables printing with {:?}

### 5. Generics
```rust
fn repl<F>(mut execute_fn: F) -> Result<()>
where
    F: FnMut(&str) -> Result<()>
```
- Write code that works with multiple types
- `F` can be any function matching the signature
- Type safety with flexibility

### 6. Options
```rust
pub fn get_column_index(&self, name: &str) -> Option<usize>
//                                            ^^^^^^^^^^^^^
```
- `Option<T>` represents a value that may or may not exist
- `Some(value)` or `None`
- Prevents null pointer errors

## 🔍 Database Concepts Explained

### 1. Indexes
**What**: Data structures that speed up queries
**How**: B-Trees keep data sorted for fast lookups
**Trade-off**: Faster reads, slower writes
**When to use**: Columns frequently used in WHERE clauses

### 2. Primary Keys
**What**: Unique identifier for each row
**Why**: Ensures data integrity
**Implementation**: Automatically indexed, no duplicates allowed

### 3. Page-Based Storage
**What**: Grouping multiple rows into fixed-size blocks
**Why**: Matches how disks actually work
**Benefit**: Read 100 rows in one disk operation vs 100 separate operations

### 4. Query Planning
**What**: Choosing the best way to execute a query
**Example**: Use index if available, otherwise scan all rows
**Real databases**: Much more complex with cost-based optimization

### 5. Sharding (Horizontal Partitioning)
**What**: Splitting data across multiple databases
**How**: Hash the key to determine which shard
**Benefits**: Scalability, performance, fault tolerance
**Challenge**: Joins across shards are expensive

### 6. ACID Properties (Partially Implemented)
- **Atomicity**: Not implemented (no transactions)
- **Consistency**: Partial (schema validation, primary keys)
- **Isolation**: Not implemented (single-threaded)
- **Durability**: Not implemented (in-memory only)

## 🎓 Learning Path

### Beginner
1. Understand the basic data types (`src/storage/mod.rs`)
2. Learn how rows and tables work
3. Run simple CREATE TABLE and INSERT commands
4. Practice SELECT queries

### Intermediate
1. Study B-Tree implementation (`src/storage/btree.rs`)
2. Understand page-based storage (`src/storage/page.rs`)
3. Create indexes and observe performance differences
4. Explore the SQL parser (`src/query/parser.rs`)

### Advanced
1. Study sharding implementation (`src/sharding/mod.rs`)
2. Modify the code to add new features:
   - Range queries (SELECT WHERE age > 25)
   - Multiple column indexes
   - JOIN operations
   - Transactions
3. Benchmark and optimize performance

## 🧪 Testing

Run the built-in tests:

```bash
cargo test
```

This will run tests for:
- B-Tree operations
- Sharding distribution
- All other components

## 📊 Example Session

```
rustydb> CREATE TABLE employees (id INTEGER PRIMARY KEY, name TEXT, salary INTEGER)
Table 'employees' created

rustydb> INSERT INTO employees VALUES (1, 'Alice', 80000)
1 row inserted into 'employees'

rustydb> INSERT INTO employees VALUES (2, 'Bob', 90000)
1 row inserted into 'employees'

rustydb> INSERT INTO employees VALUES (3, 'Charlie', 75000)
1 row inserted into 'employees'

rustydb> SELECT * FROM employees
┌────┬─────────┬────────┐
│ id │ name    │ salary │
├────┼─────────┼────────┤
│ 1  │ Alice   │ 80000  │
│ 2  │ Bob     │ 90000  │
│ 3  │ Charlie │ 75000  │
└────┴─────────┴────────┘

3 row(s) returned

rustydb> CREATE INDEX ON employees (name)
Index created on 'employees.name'

rustydb> SELECT * FROM employees WHERE name = 'Bob'
┌────┬──────┬────────┐
│ id │ name │ salary │
├────┼──────┼────────┤
│ 2  │ Bob  │ 90000  │
└────┴──────┴────────┘

1 row(s) returned

rustydb> UPDATE employees SET salary = 95000 WHERE id = 2
1 row(s) updated in 'employees'

rustydb> .exit
Goodbye!
```

## 🚀 Next Steps & Extensions

Want to learn more? Try implementing:

1. **Transactions**: BEGIN, COMMIT, ROLLBACK
2. **JOIN Operations**: SELECT from multiple tables
3. **Aggregations**: COUNT, SUM, AVG, MIN, MAX
4. **Persistence**: Write data to disk
5. **Concurrency**: Multi-threaded query execution
6. **Query Optimizer**: Cost-based query planning
7. **More Data Types**: DATE, TIMESTAMP, BLOB
8. **Constraints**: FOREIGN KEY, UNIQUE, CHECK

## 📚 Additional Resources

### Learn Rust
- [The Rust Book](https://doc.rust-lang.org/book/)
- [Rust by Example](https://doc.rust-lang.org/rust-by-example/)

### Learn Databases
- [CMU Database Course](https://15445.courses.cs.cmu.edu/)
- [Database Internals Book](https://www.databass.dev/)

## 📝 License

This is an educational project. Feel free to use and modify for learning purposes!

## 🤝 Contributing

This is a learning project! Feel free to:
- Add features
- Improve documentation
- Fix bugs
- Share your learning experience

---

**Happy Learning! 🦀📊**
