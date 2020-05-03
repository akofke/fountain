use criterion::{Criterion, BenchmarkId, criterion_main, criterion_group};
use raytracer::blocked_array::BlockedArray;
use rand::{thread_rng, Rng, FromEntropy};
use rand::distributions::{Standard, Uniform};
use rand::prelude::SmallRng;
use raytracer::spectrum::Spectrum;

fn bench(c: &mut Criterion) {

    let mut group = c.benchmark_group("ArrayLocalSum");
    let sizes = [8, 64, 246, 512, 1000, 2000];
    for size in &sizes {
        group.bench_with_input(BenchmarkId::new("Flat", size), size, |b, &size| {
            let array = gen_flat(size);
            let mut rng = SmallRng::from_entropy();
            let distr = Uniform::new_inclusive(0, size - 2);

            b.iter(|| {
                let coords = get_coords(size, &mut rng, &distr);
                coords.iter()
                    .map(|&(x, y)| array[y * size + x])
                    .sum::<Spectrum>()
            });
        });

        group.bench_with_input(BenchmarkId::new("Blocked", size), size, |b, &size| {
            let array = gen_blocked(size);
            let mut rng = SmallRng::from_entropy();
            let distr = Uniform::new_inclusive(0, size - 2);

            b.iter(|| {
                let coords = get_coords(size, &mut rng, &distr);
                coords.iter()
                    .map(|&(x, y)| array[(x, y)])
                    .sum::<Spectrum>()
            });
        });
    }

}

fn gen_blocked(size: usize) -> BlockedArray<Spectrum, 2> {
    let vec = gen_flat(size);
    BlockedArray::with_default_block_size(&vec, size, size)
}

fn gen_flat(size: usize) -> Vec<Spectrum> {
    let len = size * size;
    let mut rng = thread_rng();

    rng.sample_iter(&Standard)
        .take(len)
        .collect::<Vec<[f32; 3]>>()
        .into_iter()
        .map(|a| Spectrum::from(a))
        .collect()
}

#[inline]
fn get_coords(size: usize, rng: &mut SmallRng, distr: &Uniform<usize>) -> [(usize, usize); 4] {
    let x = rng.sample(distr);
    let y = rng.sample(distr);
    [
        (x, y),
        (x + 1, y),
        (x, y + 1),
        (x + 1, y + 1)
    ]
}

criterion_group!(benches, bench);
criterion_main!(benches);
