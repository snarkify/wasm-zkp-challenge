use wasm_zkp_challenge::msm::{generate_msm_inputs, compute_msm};
use criterion::{BenchmarkId, Criterion, black_box, criterion_group, criterion_main, Throughput};

fn bench_pippenger_msm(c: &mut Criterion) {
    let mut group = c.benchmark_group("bench_pippenger_msm");
    for size in [8, 10, 12, 14].iter() {
        let (point_vec, scalar_vec) = generate_msm_inputs(1<<size);
        // I don't think blakc_box is needed based on what I am reading in the docs. Shouldn't
        // really hurt anything thought, so I'll just leave it.
        let point_vec = black_box(point_vec);
        let scalar_vec = black_box(scalar_vec);
        let input = (point_vec, scalar_vec);

        group.throughput(Throughput::Elements(1<<size));
        group.bench_with_input(
            BenchmarkId::from_parameter(format!("input vector length: 2^{}", size)),
            &input,
            |b, input| {
                b.iter(|| {
                    let _res = compute_msm(input.0.clone(), input.1.clone());
                })
            }
        );
    }
}

criterion_group!(benches, bench_pippenger_msm);
criterion_main!(benches);
