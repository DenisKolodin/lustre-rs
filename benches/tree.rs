use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use lustre::{scenes, tree::Tree};
use rand::SeedableRng;

fn bench_gen(c: &mut Criterion) {
    // configuration of criterion
    let mut bench_group = c.benchmark_group("tree_gen");
    // filter noise more noise
    bench_group.noise_threshold(0.05);
    // smaller sig level to combat noise
    bench_group.significance_level(0.1);
    // more samples -> more precision
    bench_group.sample_size(1000);

    // modify scene selection here
    let scenes_to_check = [scenes::SceneType::CoverPhoto, scenes::SceneType::CornellBox];

    // check against each chosen scene
    for scene in scenes_to_check {
        // configuration of Tree input
        let scene_name = format!("{scene:?}");
        let mut rng = rand::rngs::SmallRng::seed_from_u64(0);
        let geo = scenes::get_geometry(scene, &mut rng, 0.0..1.0);

        // use bench with input for cleaner per-scene test name
        bench_group.bench_with_input(BenchmarkId::from_parameter(scene_name), &geo, |b, s| {
            // no need for iter_batched since we don't modify the input
            b.iter(|| Tree::new(s.clone(), 0.0, 1.0))
        });
    }

    bench_group.finish();
}

criterion_group! {benches, bench_gen}
criterion_main!(benches);
