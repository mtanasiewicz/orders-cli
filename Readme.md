# Orders CLI
This is a Rust CLI application that analyzes order data from CSV files and generates statistics.

## Build and Test Commands

```bash
cargo build                  # Compile the project
cargo run -- orders.csv      # Run with a CSV file
cargo test                   # Run all tests
cargo test <testname>        # Run specific test by name
cargo test order::tests      # Run tests in a specific module
```

## Architecture

### Module Structure

- **main.rs** - CLI entry point using clap for argument parsing
- **order.rs** - `Order` struct and `OrderStatus` enum with CSV line parsing (`Order::from_str`)
- **reader.rs** - CSV file reading with `read_csv()` and generic `read_from()` for any `BufRead`
- **statistics.rs** - Extensible statistics framework using the `Stat` trait
- **statistics/amount_by_status.rs** - Concrete `Stat` implementation for amount aggregation

### Extensibility Pattern

Statistics use a trait-based design for extensibility:

```rust
pub trait Stat {
    fn accept(&mut self, order: &Order);
    fn render(&self, f: &mut Formatter) -> fmt::Result;
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
