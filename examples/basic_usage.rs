// Example: Basic database usage
// Run with: cargo run --example basic_usage

use rustydb::{QueryExecutor, QueryParser};

fn main() -> anyhow::Result<()> {
    println!("=== RustyDB Basic Usage Example ===\n");

    // Create a new database executor
    let mut db = QueryExecutor::new();

    // 1. Create a table
    println!("1. Creating a 'users' table...");
    let sql = "CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT, age INTEGER, active BOOLEAN)";
    let query = QueryParser::parse(sql)?;
    let result = db.execute(query)?;
    println!("{}\n", result.format());

    // 2. Insert some data
    println!("2. Inserting users...");
    let inserts = vec![
        "INSERT INTO users VALUES (1, 'Alice Johnson', 30, true)",
        "INSERT INTO users VALUES (2, 'Bob Smith', 25, true)",
        "INSERT INTO users VALUES (3, 'Charlie Brown', 35, false)",
        "INSERT INTO users VALUES (4, 'Diana Prince', 28, true)",
        "INSERT INTO users VALUES (5, 'Eve Adams', 32, true)",
    ];

    for sql in inserts {
        let query = QueryParser::parse(sql)?;
        let result = db.execute(query)?;
        println!("{}", result.format());
    }
    println!();

    // 3. Query all users
    println!("3. Selecting all users...");
    let query = QueryParser::parse("SELECT * FROM users")?;
    let result = db.execute(query)?;
    println!("{}\n", result.format());

    // 4. Create an index
    println!("4. Creating an index on 'name' column...");
    let query = QueryParser::parse("CREATE INDEX ON users (name)")?;
    let result = db.execute(query)?;
    println!("{}\n", result.format());

    // 5. Query with WHERE clause (uses index)
    println!("5. Selecting user with name = 'Bob Smith' (uses index)...");
    let query = QueryParser::parse("SELECT * FROM users WHERE name = 'Bob Smith'")?;
    let result = db.execute(query)?;
    println!("{}\n", result.format());

    // 6. Update a user
    println!("6. Updating Bob's age to 26...");
    let query = QueryParser::parse("UPDATE users SET age = 26 WHERE id = 2")?;
    let result = db.execute(query)?;
    println!("{}\n", result.format());

    // 7. Verify the update
    println!("7. Verifying the update...");
    let query = QueryParser::parse("SELECT * FROM users WHERE id = 2")?;
    let result = db.execute(query)?;
    println!("{}\n", result.format());

    // 8. Delete a user
    println!("8. Deleting user with id = 3...");
    let query = QueryParser::parse("DELETE FROM users WHERE id = 3")?;
    let result = db.execute(query)?;
    println!("{}\n", result.format());

    // 9. Show final state
    println!("9. Final state of the table...");
    let query = QueryParser::parse("SELECT * FROM users")?;
    let result = db.execute(query)?;
    println!("{}\n", result.format());

    println!("=== Example Complete ===");
    Ok(())
}
