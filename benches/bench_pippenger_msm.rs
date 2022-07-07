use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use wasm_zkp_challenge::msm::{read_or_generate_instances};
use std::path::{Path, PathBuf};
mod perf;

const TEST_DIR_BASE: &'static str = "./.test";

fn bench_instance_path(count: usize, k: usize) -> PathBuf {
    Path::new(TEST_DIR_BASE).join(format!("{}x{}", count, k)).join("instances")
}

fn bench_pippenger_msm(c: &mut Criterion) {
    let mut group = c.benchmark_group("bench_pippenger_msm");
    for k in [8, 10, 12, 14].iter() {
        let path = bench_instance_path(1, *k);
        let instances = read_or_generate_instances(&path, 1, 1 << k).unwrap();
        // I don't think black_box is needed based on what I am reading in the docs.
        // Shouldn't really hurt anything though, so I'll just leave it.
        let input = black_box(&instances[0]);

        group.throughput(Throughput::Elements(1 << k));
        group.bench_with_input(
            BenchmarkId::from_parameter(format!("input_vector_length_2_{}", k)),
            &input,
            |b, input| {
                b.iter(|| {
                    let _res = input.compute_msm();
                })
            },
        );
    }
}

criterion_group! {
    name = benches;
    config = Criterion::default().with_profiler(perf::FlamegraphProfiler::new(100));
    targets = bench_pippenger_msm
}
criterion_main!(benches);
