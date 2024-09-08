use rand::SeedableRng;
use rand_pcg::Pcg64Mcg;
use js_sys::Math::random;

pub fn rng_with_random_seed() -> Pcg64Mcg {
  // TODO: improve random seed getting
  Pcg64Mcg::seed_from_u64((random() * std::u64::MAX as f64) as u64)
}