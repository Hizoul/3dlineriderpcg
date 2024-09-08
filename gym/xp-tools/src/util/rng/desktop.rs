use rand::SeedableRng;
use rand_pcg::Pcg64Mcg;
pub fn rng_with_random_seed() -> Pcg64Mcg {  
  Pcg64Mcg::from_entropy()
}
