//! Benchmarks for output sinks (CSV, JSON).
//!
//! Tests the throughput of writing journal entries to different formats.

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use std::io::Write;
use tempfile::NamedTempFile;

use datasynth_core::traits::Sink;
use datasynth_output::{CsvSink, JsonLinesSink};

mod common;
use common::generate_entries;

/// Benchmark CSV sink writing at different batch sizes.
fn bench_csv_sink(c: &mut Criterion) {
    let mut group = c.benchmark_group("csv_sink");

    for batch_size in [100, 1_000, 10_000].iter() {
        let entries = generate_entries(*batch_size);

        group.throughput(Throughput::Elements(*batch_size as u64));

        group.bench_with_input(
            BenchmarkId::from_parameter(batch_size),
            &entries,
            |b, entries| {
                b.iter_with_setup(
                    || {
                        let temp_file = NamedTempFile::new().unwrap();
                        CsvSink::new(temp_file.path().to_path_buf()).unwrap()
                    },
                    |mut sink| {
                        for entry in entries.iter().cloned() {
                            sink.write(entry).unwrap();
                        }
                        sink.flush().unwrap();
                    },
                );
            },
        );
    }

    group.finish();
}

/// Benchmark JSON Lines sink writing at different batch sizes.
fn bench_json_sink(c: &mut Criterion) {
    let mut group = c.benchmark_group("json_lines_sink");

    for batch_size in [100, 1_000, 10_000].iter() {
        let entries = generate_entries(*batch_size);

        group.throughput(Throughput::Elements(*batch_size as u64));

        group.bench_with_input(
            BenchmarkId::from_parameter(batch_size),
            &entries,
            |b, entries| {
                b.iter_with_setup(
                    || {
                        let temp_file = NamedTempFile::new().unwrap();
                        JsonLinesSink::new(temp_file.path().to_path_buf()).unwrap()
                    },
                    |mut sink| {
                        for entry in entries.iter().cloned() {
                            sink.write(entry).unwrap();
                        }
                        sink.flush().unwrap();
                    },
                );
            },
        );
    }

    group.finish();
}

/// Benchmark JSON serialization only (no I/O).
fn bench_json_serialization(c: &mut Criterion) {
    let mut group = c.benchmark_group("json_serialization");

    for batch_size in [100, 1_000, 10_000].iter() {
        let entries = generate_entries(*batch_size);

        group.throughput(Throughput::Elements(*batch_size as u64));

        group.bench_with_input(
            BenchmarkId::from_parameter(batch_size),
            &entries,
            |b, entries| {
                b.iter(|| {
                    for entry in entries {
                        black_box(serde_json::to_string(entry).unwrap());
                    }
                });
            },
        );
    }

    group.finish();
}

/// Benchmark CSV formatting only (no I/O).
fn bench_csv_formatting(c: &mut Criterion) {
    let mut group = c.benchmark_group("csv_formatting");

    for batch_size in [100, 1_000, 10_000].iter() {
        let entries = generate_entries(*batch_size);

        group.throughput(Throughput::Elements(*batch_size as u64));

        group.bench_with_input(
            BenchmarkId::from_parameter(batch_size),
            &entries,
            |b, entries| {
                b.iter(|| {
                    let mut buffer = Vec::with_capacity(1024 * 1024);
                    for entry in entries {
                        for line in &entry.lines {
                            writeln!(
                                buffer,
                                "{},{},{},{},{},{},{},{:?},{},{},{},{}",
                                entry.header.document_id,
                                entry.header.company_code,
                                entry.header.fiscal_year,
                                entry.header.fiscal_period,
                                entry.header.posting_date,
                                entry.header.document_type,
                                entry.header.currency,
                                entry.header.source,
                                line.line_number,
                                line.gl_account,
                                line.debit_amount,
                                line.credit_amount,
                            )
                            .unwrap();
                        }
                    }
                    black_box(buffer)
                });
            },
        );
    }

    group.finish();
}

/// Benchmark line item throughput (lines per second).
fn bench_line_throughput(c: &mut Criterion) {
    let mut group = c.benchmark_group("line_throughput");

    let entries = generate_entries(10_000);
    let total_lines: u64 = entries.iter().map(|e| e.line_count() as u64).sum();

    group.throughput(Throughput::Elements(total_lines));

    // CSV line throughput
    group.bench_function("csv", |b| {
        b.iter_with_setup(
            || {
                let temp_file = NamedTempFile::new().unwrap();
                CsvSink::new(temp_file.path().to_path_buf()).unwrap()
            },
            |mut sink| {
                for entry in entries.iter().cloned() {
                    sink.write(entry).unwrap();
                }
                sink.flush().unwrap();
            },
        );
    });

    // JSON line throughput
    group.bench_function("json", |b| {
        b.iter_with_setup(
            || {
                let temp_file = NamedTempFile::new().unwrap();
                JsonLinesSink::new(temp_file.path().to_path_buf()).unwrap()
            },
            |mut sink| {
                for entry in entries.iter().cloned() {
                    sink.write(entry).unwrap();
                }
                sink.flush().unwrap();
            },
        );
    });

    group.finish();
}

/// Compare CSV vs JSON sink performance.
fn bench_sink_comparison(c: &mut Criterion) {
    let mut group = c.benchmark_group("sink_comparison");
    let entries = generate_entries(5_000);

    group.throughput(Throughput::Elements(5_000));

    group.bench_function("csv_sink", |b| {
        b.iter_with_setup(
            || {
                let temp_file = NamedTempFile::new().unwrap();
                CsvSink::new(temp_file.path().to_path_buf()).unwrap()
            },
            |mut sink| {
                for entry in entries.iter().cloned() {
                    sink.write(entry).unwrap();
                }
                sink.close().unwrap();
            },
        );
    });

    group.bench_function("json_sink", |b| {
        b.iter_with_setup(
            || {
                let temp_file = NamedTempFile::new().unwrap();
                JsonLinesSink::new(temp_file.path().to_path_buf()).unwrap()
            },
            |mut sink| {
                for entry in entries.iter().cloned() {
                    sink.write(entry).unwrap();
                }
                sink.close().unwrap();
            },
        );
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_csv_sink,
    bench_json_sink,
    bench_json_serialization,
    bench_csv_formatting,
    bench_line_throughput,
    bench_sink_comparison,
);

criterion_main!(benches);
