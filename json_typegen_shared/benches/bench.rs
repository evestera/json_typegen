use criterion::{Criterion, criterion_group, criterion_main};
use json_typegen_shared::{Options, codegen};
use std::hint::black_box;

fn codegen_benchmark(c: &mut Criterion) {
    c.bench_function("magic_card_list", |b| {
        b.iter(|| {
            codegen(
                "Cards",
                black_box(include_str!("fixtures/magic_card_list.json")),
                Options::default(),
            )
        })
    });

    c.bench_function("zalando_article", |b| {
        b.iter(|| {
            codegen(
                "Article",
                black_box(include_str!("fixtures/zalando_article.json")),
                Options::default(),
            )
        })
    });
}

criterion_group!(benches, codegen_benchmark);
criterion_main!(benches);
