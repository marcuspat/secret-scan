use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use secretscan::Scanner;
use std::fs;
use tempfile::TempDir;

fn generate_test_repo(size: usize) -> TempDir {
    let temp_dir = TempDir::new().unwrap();
    let base_path = temp_dir.path();

    // Create directory structure
    for i in 0..size {
        let dir_path = base_path.join(format!("dir_{}", i % 100));
        fs::create_dir_all(&dir_path).unwrap();

        let file_path = dir_path.join(format!("file_{}.txt", i));

        // Mix of clean and secret-containing files
        let content = if i % 10 == 0 {
            // 10% chance of having secrets
            format!("const API_KEY = \"sk-1234567890abcdef1234567890abcdef\";\n// File {} content\nfunction doSomething() {{\n  return 'hello world';\n}}", i)
        } else if i % 20 == 0 {
            // 5% chance of having JWT tokens
            format!("Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIiwibmFtZSI6IkpvaG4gRG9lIiwiaWF0IjoxNTE2MjM5MDIyfQ.SflKxwRJSMeKKF2QT4fwpMeJf36POk6yJV_adQssw5c\n// File {} content", i)
        } else {
            // 85% clean files
            format!("// File {} content\nfunction doSomething() {{\n  const x = {};\n  return x * 2;\n}}", i, i)
        };

        fs::write(&file_path, content).unwrap();
    }

    temp_dir
}

fn benchmark_scanning_1k(c: &mut Criterion) {
    let temp_dir = generate_test_repo(1000);
    let scanner = Scanner::new().unwrap();

    c.bench_function("scan_1k_files", |b| {
        b.iter(|| scanner.scan_directory(black_box(temp_dir.path())).unwrap())
    });
}

fn benchmark_scanning_10k(c: &mut Criterion) {
    let temp_dir = generate_test_repo(10000);
    let scanner = Scanner::new().unwrap();

    c.bench_function("scan_10k_files", |b| {
        b.iter(|| scanner.scan_directory(black_box(temp_dir.path())).unwrap())
    });
}

fn benchmark_scanning_100k(c: &mut Criterion) {
    let temp_dir = generate_test_repo(100000);
    let scanner = Scanner::new().unwrap();

    c.bench_function("scan_100k_files", |b| {
        b.iter(|| scanner.scan_directory(black_box(temp_dir.path())).unwrap())
    });
}

fn benchmark_scaling(c: &mut Criterion) {
    let mut group = c.benchmark_group("scanning_scaling");

    for size in [100, 500, 1000, 5000, 10000].iter() {
        let temp_dir = generate_test_repo(*size);
        let scanner = Scanner::new().unwrap();

        group.bench_with_input(BenchmarkId::new("files", size), size, |b, _size| {
            b.iter(|| scanner.scan_directory(black_box(temp_dir.path())).unwrap())
        });
    }
    group.finish();
}

fn benchmark_file_sizes(c: &mut Criterion) {
    let mut group = c.benchmark_group("file_sizes");

    for file_size in [1024, 10240, 102400, 1024000].iter() {
        // 1KB, 10KB, 100KB, 1MB
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("large_file.txt");

        // Generate large file with some secrets
        let mut content = String::new();
        let secret_line = "const SECRET = \"sk-1234567890abcdef1234567890abcdef\";\n";
        let normal_line = "const normalVar = 'some normal content here';\n";

        let lines_needed = file_size / normal_line.len();
        for i in 0..lines_needed {
            if i % 100 == 0 {
                content.push_str(secret_line);
            } else {
                content.push_str(normal_line);
            }
        }

        fs::write(&file_path, content).unwrap();
        let scanner = Scanner::new().unwrap();

        group.bench_with_input(
            BenchmarkId::new("bytes", file_size),
            file_size,
            |b, _file_size| b.iter(|| scanner.scan_directory(black_box(temp_dir.path())).unwrap()),
        );
    }
    group.finish();
}

criterion_group!(
    benches,
    benchmark_scanning_1k,
    benchmark_scanning_10k,
    benchmark_scanning_100k,
    benchmark_scaling,
    benchmark_file_sizes
);
criterion_main!(benches);
