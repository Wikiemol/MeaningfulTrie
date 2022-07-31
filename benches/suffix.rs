use std::iter;

use criterion::{criterion_group, criterion_main};

use criterion::BenchmarkId;
use criterion::Criterion;
use criterion::Throughput;

use rand::{self, Rng};
use rand::distributions::{Alphanumeric, Uniform, Standard};


use trie::trie::Trie;


/// TODO: This benchmark could be improved by generating
/// strings with more structure. E.g. generating strings
/// from a markov chain or a context free grammar 
/// Curently it just generates a uniformly random string
fn suffix(c: &mut Criterion) {
    static KB: usize = 1024;

    let mut group = c.benchmark_group("suffix_trie");
    for size in [KB, 2 * KB, 4 * KB, 8 * KB, 16 * KB, 32 * KB].iter() {
        group.throughput(Throughput::Bytes(*size as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            b.iter(|| {
                Trie::suffix(&rand::thread_rng()
                    .sample_iter(&Alphanumeric)
                    .take(size)
                    .map(|x| x as char)
                    .collect::<String>(), Some(100))
            });
        });
    }
    group.finish();
}

criterion_group!(benches, suffix);
criterion_main!(benches);
