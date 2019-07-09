#[macro_use]
extern crate criterion;
#[cfg(feature = "cpu-profile")]
extern crate gperftools;

use criterion::{black_box, Criterion, ParameterizedBenchmark};
use storage_proofs::drgraph::Graph;
use storage_proofs::hasher::blake2s::Blake2sHasher;
use storage_proofs::hasher::pedersen::PedersenHasher;
use storage_proofs::hasher::sha256::Sha256Hasher;
use storage_proofs::hasher::Hasher;
use storage_proofs::zigzag_graph::{ZigZag, ZigZagBucketGraph};

#[cfg(feature = "cpu-profile")]
#[inline(always)]
fn start_profile(stage: &str) {
    gperftools::profiler::PROFILER
        .lock()
        .unwrap()
        .start(format!("./{}.profile", stage))
        .unwrap();
}

#[cfg(not(feature = "cpu-profile"))]
#[inline(always)]
fn start_profile(_stage: &str) {}

#[cfg(feature = "cpu-profile")]
#[inline(always)]
fn stop_profile() {
    gperftools::profiler::PROFILER
        .lock()
        .unwrap()
        .stop()
        .unwrap();
}

#[cfg(not(feature = "cpu-profile"))]
#[inline(always)]
fn stop_profile() {}

fn pregenerate_graph<AH, BH>(size: usize) -> ZigZagBucketGraph<AH, BH>
where
    AH: Hasher,
    BH: Hasher,
{
    let seed = [1, 2, 3, 4, 5, 6, 7];
    ZigZagBucketGraph::<AH, BH>::new_zigzag(size, 5, 8, seed)
}

fn parents_loop<AH, BH, G>(graph: &G, parents: &mut [usize])
where
    AH: Hasher,
    BH: Hasher,
    G: Graph<AH, BH>,
{
    (0..graph.size())
        .map(|node| graph.parents(node, parents))
        .collect()
}

fn parents_loop_benchmark(cc: &mut Criterion) {
    let sizes = vec![10, 50, 1000];

    cc.bench(
        "parents in a loop",
        ParameterizedBenchmark::new(
            "Blake2s",
            |b, size| {
                let graph = pregenerate_graph::<Blake2sHasher, Blake2sHasher>(*size);
                let mut parents = vec![0; graph.degree()];
                start_profile(&format!("parents-blake2s-{}", *size));
                b.iter(|| {
                    black_box(parents_loop::<Blake2sHasher, Blake2sHasher, _>(
                        &graph,
                        &mut parents,
                    ))
                });
                stop_profile();
            },
            sizes,
        )
        .with_function("Pedersen", |b, degree| {
            let graph = pregenerate_graph::<PedersenHasher, PedersenHasher>(*degree);
            let mut parents = vec![0; graph.degree()];
            b.iter(|| {
                black_box(parents_loop::<PedersenHasher, PedersenHasher, _>(
                    &graph,
                    &mut parents,
                ))
            })
        })
        .with_function("Sha256", |b, degree| {
            let graph = pregenerate_graph::<Sha256Hasher, Sha256Hasher>(*degree);
            let mut parents = vec![0; graph.degree()];
            b.iter(|| {
                black_box(parents_loop::<Sha256Hasher, Sha256Hasher, _>(
                    &graph,
                    &mut parents,
                ))
            })
        }),
    );
}

criterion_group!(benches, parents_loop_benchmark);
criterion_main!(benches);
