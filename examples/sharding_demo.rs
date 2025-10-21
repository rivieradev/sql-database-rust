// Example: Sharding demonstration
// Run with: cargo run --example sharding_demo

use rustydb::ShardedDatabase;

fn main() -> anyhow::Result<()> {
    println!("=== RustyDB Sharding Demo ===\n");

    // Create a sharded database with 4 shards
    let num_shards = 4;
    let mut db = ShardedDatabase::new(num_shards);

    println!("Created database with {} shards\n", num_shards);

    // 1. Create a table (created on all shards)
    println!("1. Creating 'products' table on all shards...");
    db.execute("CREATE TABLE products (id INTEGER PRIMARY KEY, name TEXT, price INTEGER, category TEXT)")?;
    println!("Table created on all shards\n");

    // 2. Insert data (distributed across shards based on hash)
    println!("2. Inserting products (will be distributed across shards)...");
    let products = vec![
        (1, "Laptop", 1200, "Electronics"),
        (2, "Mouse", 25, "Electronics"),
        (3, "Desk", 300, "Furniture"),
        (4, "Chair", 150, "Furniture"),
        (5, "Monitor", 400, "Electronics"),
        (6, "Keyboard", 80, "Electronics"),
        (7, "Lamp", 45, "Furniture"),
        (8, "Bookshelf", 200, "Furniture"),
        (9, "Webcam", 120, "Electronics"),
        (10, "Headphones", 90, "Electronics"),
    ];

    for (id, name, price, category) in products {
        let sql = format!(
            "INSERT INTO products VALUES ({}, '{}', {}, '{}')",
            id, name, price, category
        );
        db.execute(&sql)?;
        println!("  Inserted: {} - ${}", name, price);
    }
    println!();

    // 3. Show shard distribution
    println!("3. Shard distribution:");
    let stats = db.get_shard_stats("products");
    for stat in &stats {
        println!("  {}", stat.format());
    }
    println!();

    // 4. Query specific item (routes to single shard)
    println!("4. Querying single product (id = 5) - routes to single shard...");
    let result = db.execute("SELECT * FROM products WHERE id = 5")?;
    println!("{}\n", result.format());

    // 5. Query all items (scatter-gather across all shards)
    println!("5. Querying all products - scatter-gather across all shards...");
    let result = db.execute("SELECT * FROM products")?;
    println!("{}\n", result.format());

    // 6. Create index on all shards
    println!("6. Creating index on 'category' column...");
    db.execute("CREATE INDEX ON products (category)")?;
    println!("Index created on all shards\n");

    // 7. Update using index
    println!("7. Updating product price (id = 2)...");
    db.execute("UPDATE products SET price = 30 WHERE id = 2")?;
    println!();

    // 8. Verify update
    println!("8. Verifying update...");
    let result = db.execute("SELECT * FROM products WHERE id = 2")?;
    println!("{}\n", result.format());

    println!("=== Sharding Concepts Demonstrated ===");
    println!("✓ Hash-based shard routing");
    println!("✓ Data distribution across shards");
    println!("✓ Single-shard queries (WHERE with specific value)");
    println!("✓ Scatter-gather queries (SELECT * without WHERE)");
    println!("✓ Index creation on all shards");
    println!("\n=== Demo Complete ===");

    Ok(())
}
