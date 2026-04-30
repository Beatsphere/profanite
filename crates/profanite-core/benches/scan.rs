//! Basic scan throughput bench. Target: ≤ 50 µs per 1 KB English input.

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use profanite_core::Profanite;

/// A ~1 KB block of mostly-benign English text with a few profanities sprinkled
/// in, so the matcher touches both the common no-match path and the
/// boundary-validation path.
const CORPUS_1KB: &str = concat!(
    "The quick brown fox jumps over the lazy dog. ",
    "She sells seashells by the seashore, and nobody on the beach said a damn word about it. ",
    "In a village of La Mancha, the name of which I have no desire to call to mind, there lived ",
    "not long since one of those gentlemen that keep a lance in the lance-rack. ",
    "It was the best of times, it was the worst of times, it was the age of wisdom, ",
    "it was the age of foolishness, it was the epoch of belief, it was the epoch of incredulity. ",
    "He muttered something under his breath about the fucking weather and walked off. ",
    "All happy families are alike; each unhappy family is unhappy in its own way. ",
    "Call me Ishmael. Some years ago—never mind how long precisely—having little or no money in ",
    "my purse, and nothing particular to interest me on shore, I thought I would sail about.",
);

fn bench_contains(c: &mut Criterion) {
    let p = Profanite::builder().build().unwrap();
    c.bench_function("contains_profanity_1kb", |b| {
        b.iter(|| black_box(p.contains_profanity(black_box(CORPUS_1KB))))
    });
}

fn bench_find(c: &mut Criterion) {
    let p = Profanite::builder().build().unwrap();
    c.bench_function("find_1kb", |b| {
        b.iter(|| black_box(p.find(black_box(CORPUS_1KB))))
    });
}

fn bench_censor(c: &mut Criterion) {
    let p = Profanite::builder().build().unwrap();
    c.bench_function("censor_1kb", |b| {
        b.iter(|| black_box(p.censor(black_box(CORPUS_1KB))))
    });
}

criterion_group!(benches, bench_contains, bench_find, bench_censor);
criterion_main!(benches);
