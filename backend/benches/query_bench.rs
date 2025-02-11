use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId};
use tokio::runtime::Runtime;

mod crdt;
use crdt::CRDT;

fn simple_query(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let mut group = c.benchmark_group("Simple Query Benchmark");

    for i in [1, 10, 100, 1000].iter() {
        group.bench_with_input(BenchmarkId::new("updates", i), i, |b, &num_updates| {
            b.to_async(&rt).iter(|| async {
                let mut crdt = CRDT::new("benchmark_user", "benchmark_password").await;
                
                // Perform `num_updates` updates
                for j in 0..num_updates {
                    crdt.update(&format!("key{}", j), &format!("value{}", j));
                }

                let _ = crdt.save().await;
            });
        });
    }

    group.finish();
}

// Criterion boilerplate
criterion_group!(benches, simple_query);
criterion_main!(benches);
