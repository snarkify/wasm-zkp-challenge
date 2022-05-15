use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use wasm_zkp_challenge::msm::{compute_msm, compute_pippenger, compute_pippenger_affine,compute_msm_affine, generate_msm_inputs};
mod perf;

fn bench_pippenger_msm(c: &mut Criterion) {
    let mut group = c.benchmark_group("bench_pippenger_msm");
    for size in [8, 10 ].iter() {
        let (point_vec, scalar_vec) = generate_msm_inputs(1 << size);
        let point_vec = black_box(point_vec);
        let scalar_vec = black_box(scalar_vec);
        let input = (point_vec, scalar_vec);

        group.bench_with_input(
            BenchmarkId::from_parameter(format!("input_vector_length_2_{}", size)),
            &input,
            |b, input| {
                b.iter(|| {
                    compute_pippenger_affine(input.0.clone(), input.1.clone());
                })
            },
        );
    }
}

criterion_group!{
    name = benches;
    config = Criterion::default().with_profiler(perf::FlamegraphProfiler::new(100));
    targets = bench_pippenger_msm
}
criterion_main!(benches);
