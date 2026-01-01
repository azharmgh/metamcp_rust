//! Benchmarks for authentication operations

use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId};
use std::hint::black_box;
use metamcp::auth::{JwtService, ApiKeyEncryption};
use uuid::Uuid;

/// Benchmark JWT token generation
fn bench_jwt_generation(c: &mut Criterion) {
    let service = JwtService::new("benchmark_secret_key_12345678901234567890");
    let api_key_id = Uuid::new_v4();

    c.bench_function("jwt_generate_token", |b| {
        b.iter(|| {
            service.generate_token(black_box(api_key_id)).unwrap()
        })
    });
}

/// Benchmark JWT token validation
fn bench_jwt_validation(c: &mut Criterion) {
    let service = JwtService::new("benchmark_secret_key_12345678901234567890");
    let api_key_id = Uuid::new_v4();
    let token = service.generate_token(api_key_id).unwrap();

    c.bench_function("jwt_validate_token", |b| {
        b.iter(|| {
            service.validate_token(black_box(&token)).unwrap()
        })
    });
}

/// Benchmark complete JWT round-trip (generate + validate)
fn bench_jwt_roundtrip(c: &mut Criterion) {
    let service = JwtService::new("benchmark_secret_key_12345678901234567890");
    let api_key_id = Uuid::new_v4();

    c.bench_function("jwt_roundtrip", |b| {
        b.iter(|| {
            let token = service.generate_token(black_box(api_key_id)).unwrap();
            service.validate_token(&token).unwrap()
        })
    });
}

/// Benchmark API key generation
fn bench_api_key_generation(c: &mut Criterion) {
    c.bench_function("api_key_generate", |b| {
        b.iter(|| {
            ApiKeyEncryption::generate_api_key()
        })
    });
}

/// Benchmark API key hashing
fn bench_api_key_hashing(c: &mut Criterion) {
    let api_key = "mcp_benchmark_test_key_12345678901234567890";

    c.bench_function("api_key_hash", |b| {
        b.iter(|| {
            ApiKeyEncryption::hash_api_key(black_box(api_key)).unwrap()
        })
    });
}

/// Benchmark API key encryption
fn bench_api_key_encryption(c: &mut Criterion) {
    let key = [0u8; 32];
    let encryption = ApiKeyEncryption::new(&key);
    let api_key = "mcp_benchmark_test_key_12345678901234567890";

    c.bench_function("api_key_encrypt", |b| {
        b.iter(|| {
            encryption.encrypt(black_box(api_key)).unwrap()
        })
    });
}

/// Benchmark API key decryption
fn bench_api_key_decryption(c: &mut Criterion) {
    let key = [0u8; 32];
    let encryption = ApiKeyEncryption::new(&key);
    let api_key = "mcp_benchmark_test_key_12345678901234567890";
    let encrypted = encryption.encrypt(api_key).unwrap();

    c.bench_function("api_key_decrypt", |b| {
        b.iter(|| {
            encryption.decrypt(black_box(&encrypted)).unwrap()
        })
    });
}

/// Benchmark API key encryption + decryption roundtrip
fn bench_api_key_roundtrip(c: &mut Criterion) {
    let key = [0u8; 32];
    let encryption = ApiKeyEncryption::new(&key);
    let api_key = "mcp_benchmark_test_key_12345678901234567890";

    c.bench_function("api_key_roundtrip", |b| {
        b.iter(|| {
            let encrypted = encryption.encrypt(black_box(api_key)).unwrap();
            encryption.decrypt(&encrypted).unwrap()
        })
    });
}

/// Benchmark with varying API key lengths
fn bench_api_key_encryption_varying_lengths(c: &mut Criterion) {
    let key = [0u8; 32];
    let encryption = ApiKeyEncryption::new(&key);

    let mut group = c.benchmark_group("api_key_encrypt_length");

    for size in [32, 64, 128, 256].iter() {
        let api_key: String = (0..*size).map(|_| 'a').collect();

        group.bench_with_input(BenchmarkId::from_parameter(size), &api_key, |b, key| {
            b.iter(|| encryption.encrypt(black_box(key)).unwrap())
        });
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_jwt_generation,
    bench_jwt_validation,
    bench_jwt_roundtrip,
    bench_api_key_generation,
    bench_api_key_hashing,
    bench_api_key_encryption,
    bench_api_key_decryption,
    bench_api_key_roundtrip,
    bench_api_key_encryption_varying_lengths,
);

criterion_main!(benches);
