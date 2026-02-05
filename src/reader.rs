use crate::order::Order;
use crate::statistics::Statistics;
use csv::ReaderBuilder;
use memmap2::Mmap;
use rayon::prelude::*;
use std::fs::File;

const CHUNK_SIZE: usize = 8192 * 1024; // 8MB chunks

pub fn read_csv(file_path: &str) -> Result<Statistics, Box<dyn std::error::Error>> {
    let file = File::open(file_path)?;
    let metadata = file.metadata()?;

    if metadata.len() == 0 {
        return Ok(Statistics::new());
    }

    let m_map = unsafe { Mmap::map(&file)? };
    let data = &m_map[..];

    let chunks = split_into_chunks(data, CHUNK_SIZE);

    let statistics = chunks
        .into_par_iter()
        .map(|(chunk, line_offset)| process_chunk(chunk, line_offset))
        .reduce(Statistics::new, |mut acc, stats| {
            acc.merge(stats);
            acc
        });

    Ok(statistics)
}

/// Splits data into chunks, respecting line boundaries.
/// Returns tuples of (chunk_slice, starting_line_number).
fn split_into_chunks(data: &[u8], target_size: usize) -> Vec<(&[u8], usize)> {
    let mut chunks = Vec::new();
    let mut start = 0;
    let mut line_number = 1;

    while start < data.len() {
        let mut end = (start + target_size).min(data.len());

        if end < data.len() {
            while end < data.len() && data[end] != b'\n' {
                end += 1;
            }
            if end < data.len() {
                end += 1;
            }
        }

        let chunk = &data[start..end];
        chunks.push((chunk, line_number));

        line_number += chunk.iter().filter(|&&b| b == b'\n').count();

        start = end;
    }

    chunks
}

fn process_chunk(chunk: &[u8], line_offset: usize) -> Statistics {
    let mut statistics = Statistics::new();

    let mut reader = ReaderBuilder::new()
        .has_headers(false)
        .from_reader(chunk);

    for (i, result) in reader.records().enumerate() {
        let line_number = line_offset + i;
        match result {
            Ok(record) => match Order::from_csv_record(&record) {
                Ok(order) => statistics.accept(order),
                Err(err) => statistics.add_error(line_number, err),
            },
            Err(_) => {
                statistics.add_error(line_number, crate::order::ParseError::InvalidFormat);
            }
        }
    }

    statistics
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::order::ParseError;

    #[test]
    fn read_csv_reads_real_file() {
        use std::fs::write;

        let path = "test_read_csv.csv";
        write(path, "1,John,10,paid\n2,Jane,20,cancelled\n").unwrap();

        let stats = read_csv(path).unwrap();
        assert_eq!(stats.error_count(), 0);

        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn read_csv_returns_error_for_missing_file() {
        let result = read_csv("nonexistent_file.csv");

        assert!(result.is_err());
    }

    #[test]
    fn read_csv_handles_invalid_lines() {
        use std::fs::write;

        let path = "test_invalid.csv";
        write(path, "1,John,10,paid\ninvalid_line\n3,Bob,-30,refunded\n4,Mike,20,unknown\n").unwrap();

        let stats = read_csv(path).unwrap();
        assert_eq!(stats.error_count(), 3);

        let errors = stats.errors();
        assert_eq!(errors[0], (2, &ParseError::InvalidFormat));
        assert_eq!(errors[1], (3, &ParseError::InvalidPrice));
        assert_eq!(errors[2], (4, &ParseError::InvalidStatus));

        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn read_csv_handles_empty_file() {
        use std::fs::write;

        let path = "test_empty.csv";
        write(path, "").unwrap();

        let stats = read_csv(path).unwrap();
        assert_eq!(stats.error_count(), 0);

        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn split_into_chunks_respects_line_boundaries() {
        let data = b"line1\nline2\nline3\n";
        let chunks = split_into_chunks(data, 8);

        for (chunk, _) in &chunks {
            assert!(chunk.is_empty() || chunk.ends_with(b"\n"));
        }
    }

    #[test]
    fn split_into_chunks_tracks_line_numbers() {
        let data = b"line1\nline2\nline3\n";
        let chunks = split_into_chunks(data, 6);

        assert_eq!(chunks[0].1, 1);
        if chunks.len() > 1 {
            assert!(chunks[1].1 > 1);
        }
    }
}
