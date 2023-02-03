use app::linear::LinearExecutionState;
use criterion::{criterion_group, criterion_main, Criterion};

fn criterion_benchmark(c: &mut Criterion) {
    let mut execstate = LinearExecutionState::default();

    c.bench_function("linear 100", |b| b.iter(|| {
        execstate.runnable.runnable[1] = true;
        execstate.store.value[1000] = 0;
        app::linear::linear_exec(&mut execstate);
        assert_eq!(execstate.store.value[1000], 1000);
    }));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
