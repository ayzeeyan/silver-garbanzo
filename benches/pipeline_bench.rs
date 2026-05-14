use criterion::{criterion_group, criterion_main, Criterion};
use std::fs;

use lunadec::lua_parser::parse_source;

fn parse_benchmark(c: &mut Criterion) {
    let source = fs::read_to_string("examples/sample_obfuscated.lua").unwrap();

    c.bench_function("parse_obfuscated", |b| {
        b.iter(|| parse_source(criterion::black_box(&source)))
    });
}

criterion_group!(benches, parse_benchmark);
criterion_main!(benches);
