#[cfg(target_arch = "wasm32")]
pub mod wasm;
#[cfg(target_arch = "wasm32")]
pub use wasm::*;
#[cfg(not(target_arch = "wasm32"))]
pub mod desktop;
#[cfg(not(target_arch = "wasm32"))]
pub use desktop::*;

pub use rand::{SeedableRng, RngCore};
pub use rand_pcg::Pcg64Mcg;
pub fn extract_seed(seed: Option<u64>) -> u64 {
  match seed {
    Some(unwrapped_seed) => {
      unwrapped_seed
    },
    None => {
      let mut rng = rng_with_random_seed();
      rng.next_u64()
    }
  }
}

/**
 * Pcg64Mcg used because of speed see https://rust-random.github.io/book/guide-rngs.html#basic-pseudo-random-number-generators-prngs
 **/
pub fn from_seed(seed: Option<u64>) -> (Pcg64Mcg, u64) {
  match seed {
    Some(unwrapped_seed) => {
      let rng = Pcg64Mcg::seed_from_u64(unwrapped_seed);
      (rng, unwrapped_seed)
    },
    None => {
      let mut rng = rng_with_random_seed();
      let return_seed = rng.next_u64();
      (rng, return_seed)
    }
  }
}