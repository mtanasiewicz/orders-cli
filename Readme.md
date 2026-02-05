# Orders CLI

A high-performance Rust CLI application that analyzes order data from CSV files and generates statistics. Features parallel processing with memory-mapped files for efficient handling of large datasets.

> **Note:** This is a personal learning project for exploring Rust concepts including ownership, parallel processing with rayon, memory-mapped I/O, and idiomatic error handling.

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

# Generate a 256MB file for performance testing
orders-cli generate large.csv --size 256
```

## Build and Test Commands

```bash
cargo build                           # Compile the project
cargo build --release                 # Compile with optimizations
cargo run -- analyze orders.csv       # Analyze a CSV file
cargo run -- generate out.csv -s 1    # Generate 1MB of test data
cargo test                            # Run all tests
cargo test <testname>                 # Run specific test by name
cargo test order::tests               # Run tests in a specific module
```

## Architecture

### Module Structure

- **main.rs** - CLI entry point using clap for argument parsing with `analyze` and `generate` subcommands
- **order.rs** - `Order` struct and `OrderStatus` enum with CSV parsing (`Order::from_csv_record`)
- **reader.rs** - Parallel CSV processing with memory-mapped files and rayon
- **generator.rs** - Random CSV data generation with configurable file size
- **statistics.rs** - Statistics aggregation with merge support for parallel processing
- **statistics/*.rs** - Individual statistic implementations:
  - `amount_by_status.rs` - Amounts aggregated by order status
  - `amount_distribution.rs` - Order amounts distribution by price ranges
  - `amount_summary.rs` - Min, max, mean, median, std deviation
  - `conversion_metrics.rs` - Paid/cancelled/refunded rates
  - `top_orders.rs` - Top N highest and lowest orders
  - `customer_risk_profile.rs` - Customer risk analysis

### Parallel Processing Architecture

The reader uses a **map-reduce** pattern for parallel CSV processing:

1. **Memory-mapped file** (`memmap2`) - File is mapped to virtual memory, OS handles page loading on demand
2. **Chunk splitting** - File is divided into ~1MB chunks at line boundaries
3. **Parallel processing** (`rayon`) - Each chunk is processed independently by a thread pool
4. **Merge** - Partial statistics from all chunks are merged into final result

```
┌─────────────┐     ┌─────────────┐     ┌─────────────┐
│   Chunk 1   │     │   Chunk 2   │     │   Chunk N   │
│  (Thread 1) │     │  (Thread 2) │     │  (Thread N) │
└──────┬──────┘     └──────┬──────┘     └──────┬──────┘
       │                   │                   │
       │    Statistics     │    Statistics     │    Statistics
       │                   │                   │
       └───────────────────┼───────────────────┘
                           │
                           ▼
                    ┌──────────────┐
                    │    Merge     │
                    │  (reduce)    │
                    └──────────────┘
                           │
                           ▼
                    Final Statistics
```

Each statistic type implements a `merge()` method that combines partial results:
- Counters and sums are added
- Min/max are compared
- Top N lists are merged and re-sorted
- HashMaps are combined key-by-key

### CSV Format

```
id,customer,amount,status
1,Alice,120.50,paid
```

- **id**: positive integer
- **customer**: string
- **amount**: non-negative float
- **status**: "paid", "cancelled", or "refunded"

### Testing

All tests are inline within each module using `#[cfg(test)]` blocks. Total: 99 tests covering parsing, statistics calculations, and merge operations.

```bash
cargo test                    # Run all 99 tests
cargo test merge              # Run only merge-related tests
cargo test statistics::      # Run statistics module tests
```

## Performance

### Benchmark Results (256MB CSV file, ~7.8M orders)

Tested on Apple M2 Pro (12 cores), averaged over 5 runs.

| Version | Time | Speedup |
|---------|------|---------|
| Single-threaded (sequential) | 2.10s | 1.0x |
| Multi-threaded (rayon + mmap) | 0.46s | **4.6x** |

### Memory Usage

| Metric | Value |
|--------|-------|
| File size | 256 MB |
| Peak memory footprint | ~350 MB |
| Maximum resident set size | ~515 MB |
| CPU time (all threads) | ~2.4s |

Memory-mapped files allow processing files larger than available RAM - the OS loads only the pages currently being accessed and can evict unused pages as needed.

### How to Reproduce

```bash
# Build release binary
cargo build --release

# Generate test file
./target/release/orders-cli generate /tmp/test_256mb.csv --size 256

# Measure execution time and memory (macOS)
/usr/bin/time -l ./target/release/orders-cli analyze /tmp/test_256mb.csv

# Filter relevant metrics
/usr/bin/time -l ./target/release/orders-cli analyze /tmp/test_256mb.csv 2>&1 \
  | grep -E "execution time|real|maximum resident|peak memory"
```

## Dependencies

- **clap** - Command line argument parsing
- **csv** - CSV parsing
- **rayon** - Data parallelism (parallel iterators)
- **memmap2** - Memory-mapped file I/O
- **comfy-table** - Pretty table output
- **strum** - Enum utilities
- **rand** - Random data generation
