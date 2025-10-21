// Main entry point for the RustyDB CLI
// This provides an interactive shell to execute SQL commands

use anyhow::Result;
use clap::Parser as ClapParser;
use rustydb::{QueryExecutor, QueryParser, ShardedDatabase};
use std::io::{self, Write};

/// RustyDB - A simple SQL database implementation in Rust
#[derive(ClapParser)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Number of shards (default: 1, no sharding)
    #[arg(short, long, default_value_t = 1)]
    shards: usize,

    /// Execute a single SQL command and exit
    #[arg(short, long)]
    execute: Option<String>,
}

fn main() -> Result<()> {
    let args = Args::parse();

    if args.shards == 1 {
        // Single database mode (no sharding)
        run_single_db(args.execute)?;
    } else {
        // Sharded database mode
        run_sharded_db(args.shards, args.execute)?;
    }

    Ok(())
}

/// Run the database in single-instance mode (no sharding)
fn run_single_db(execute_cmd: Option<String>) -> Result<()> {
    let mut executor = QueryExecutor::new();

    // If a command was provided, execute it and exit
    if let Some(sql) = execute_cmd {
        execute_query(&mut executor, &sql)?;
        return Ok(());
    }

    // Interactive REPL (Read-Eval-Print Loop)
    println!("╔════════════════════════════════════════════╗");
    println!("║         RustyDB Interactive Shell         ║");
    println!("║      A Simple SQL Database in Rust        ║");
    println!("╚════════════════════════════════════════════╝");
    println!();
    println!("Type SQL commands or '.help' for help");
    println!("Type '.exit' to quit");
    println!();

    repl(|sql| execute_query(&mut executor, sql))
}

/// Run the database in sharded mode
fn run_sharded_db(num_shards: usize, execute_cmd: Option<String>) -> Result<()> {
    let mut sharded_db = ShardedDatabase::new(num_shards);

    println!("╔════════════════════════════════════════════╗");
    println!("║    RustyDB Interactive Shell (SHARDED)    ║");
    println!("╚════════════════════════════════════════════╝");
    println!();
    println!("Running with {} shards", num_shards);
    println!("Type SQL commands or '.help' for help");
    println!("Type '.exit' to quit");
    println!();

    // If a command was provided, execute it and exit
    if let Some(sql) = execute_cmd {
        execute_sharded_query(&mut sharded_db, &sql)?;
        return Ok(());
    }

    repl(|sql| execute_sharded_query(&mut sharded_db, sql))
}

/// REPL (Read-Eval-Print Loop) implementation
/// This is a common pattern for interactive shells
///
/// The 'F' is a generic type parameter - it can be any function
/// that takes a &str and returns a Result<()>
fn repl<F>(mut execute_fn: F) -> Result<()>
where
    F: FnMut(&str) -> Result<()>,
{
    let stdin = io::stdin();
    let mut stdout = io::stdout();

    loop {
        // Print prompt
        print!("rustydb> ");
        stdout.flush()?; // Ensure prompt is displayed immediately

        // Read user input
        let mut input = String::new();
        stdin.read_line(&mut input)?;

        // Trim whitespace
        let input = input.trim();

        // Handle empty input
        if input.is_empty() {
            continue;
        }

        // Handle special commands (starting with .)
        if input.starts_with('.') {
            match input {
                ".exit" | ".quit" => {
                    println!("Goodbye!");
                    break;
                }
                ".help" => {
                    print_help();
                    continue;
                }
                _ => {
                    println!("Unknown command: {}", input);
                    println!("Type '.help' for help");
                    continue;
                }
            }
        }

        // Execute the SQL query
        if let Err(e) = execute_fn(input) {
            eprintln!("Error: {}", e);
        }
    }

    Ok(())
}

/// Execute a query on a single database
fn execute_query(executor: &mut QueryExecutor, sql: &str) -> Result<()> {
    let query = QueryParser::parse(sql)?;
    let result = executor.execute(query)?;
    println!("{}", result.format());
    Ok(())
}

/// Execute a query on a sharded database
fn execute_sharded_query(db: &mut ShardedDatabase, sql: &str) -> Result<()> {
    let result = db.execute(sql)?;
    println!("{}", result.format());
    Ok(())
}

/// Print help information
fn print_help() {
    println!("╔════════════════════════════════════════════╗");
    println!("║              RustyDB Help                  ║");
    println!("╚════════════════════════════════════════════╝");
    println!();
    println!("Special Commands:");
    println!("  .help              Show this help message");
    println!("  .exit, .quit       Exit the shell");
    println!();
    println!("Supported SQL Commands:");
    println!();
    println!("  CREATE TABLE:");
    println!("    CREATE TABLE users (");
    println!("      id INTEGER PRIMARY KEY,");
    println!("      name TEXT,");
    println!("      age INTEGER");
    println!("    )");
    println!();
    println!("  INSERT:");
    println!("    INSERT INTO users VALUES (1, 'Alice', 30)");
    println!();
    println!("  SELECT:");
    println!("    SELECT * FROM users");
    println!("    SELECT * FROM users WHERE id = 1");
    println!();
    println!("  UPDATE:");
    println!("    UPDATE users SET age = 31 WHERE id = 1");
    println!();
    println!("  DELETE:");
    println!("    DELETE FROM users WHERE id = 1");
    println!();
    println!("  CREATE INDEX:");
    println!("    CREATE INDEX ON users (name)");
    println!();
    println!("Notes:");
    println!("  - All SQL keywords are case-insensitive");
    println!("  - String values must be in single quotes");
    println!("  - UPDATE and DELETE require WHERE clause");
    println!();
}
