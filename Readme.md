# Orders CLI
This is a Rust CLI application that analyzes order data from CSV files and generates statistics.

## Usage

```bash
# Analyze orders from a CSV file
orders-cli analyze <file>

# Generate a CSV file with random order data
orders-cli generate <output> --size <megabytes>
```

### Examples

```bash
# Analyze an existing orders file
orders-cli analyze orders.csv

# Generate a 10MB test file with random orders
orders-cli generate test_orders.csv --size 10

# Generate a small 0.5MB file for quick testing
orders-cli generate small.csv --size 0.5
```

## Build and Test Commands

```bash
cargo build                           # Compile the project
cargo run -- analyze orders.csv       # Analyze a CSV file
cargo run -- generate out.csv -s 1    # Generate 1MB of test data
cargo test                            # Run all tests
cargo test <testname>                 # Run specific test by name
cargo test order::tests               # Run tests in a specific module
```

## Architecture

### Module Structure

- **main.rs** - CLI entry point using clap for argument parsing with `analyze` and `generate` subcommands
- **order.rs** - `Order` struct and `OrderStatus` enum with CSV line parsing (`Order::from_str`)
- **reader.rs** - CSV file reading with `read_csv()` and generic `read_from()` for any `BufRead`
- **generator.rs** - Random CSV data generation with configurable file size
- **statistics.rs** - Extensible statistics framework using the `Stat` trait
- **statistics/amount_by_status.rs** - Concrete `Stat` implementation for amount aggregation

### Extensibility Pattern

Statistics use a trait-based design for extensibility:

```rust
trait Stat: Display {
    fn accept(&mut self, order: &Order);
}
```

To add new statistics:
1. Create a new struct implementing `Stat`
2. Add `Box::new(YourStat::new())` to the vector in `Statistics::new()`

### CSV Format

```
id,customer,amount,status
1,Alice,120.50,paid
```

- **id**: positive integer
- **customer**: string
- **amount**: non-negative float
- **status**: "paid", "cancelled", or "refunded" (case-insensitive)

### Testing

All tests are inline within each module using `#[cfg(test)]` blocks. The reader module uses `Cursor` to test file reading without real files.
