use rand::{RngCore, Rng, SeedableRng, rngs::mock::StepRng};
use rand_pcg::{Pcg64, Pcg32};
use rand_xoshiro::SplitMix64;
use xp_tools::rng::{extract_seed, from_seed};
use crate::util::py_rng::openai_gym_rng;
use mt19937::gen_res53 as next_f64;

#[derive(Copy, Clone)]
pub enum RngType {
  Pcg64Mcg,
  Pcg64,
  Pcg32,
  Mock,
  SplitMix64,
  Mt19937
}

pub type UniRng = Box<dyn RngCore>;

pub fn get_rng_for_type(rng_type_opt: &Option<RngType>, seed: Option<u64>) -> (UniRng, u64) {
  let rng_type = rng_type_opt.unwrap_or(RngType::Pcg64Mcg);
  let rng: (UniRng, u64) = match rng_type {
    RngType::Mt19937 => {
      let (rng, seed) = openai_gym_rng(seed);
      (Box::new(rng), seed)
    },
    RngType::Mock => {
      let (mut rng_helper, extracted_seed) = from_seed(seed);
      let rng = StepRng::new(rng_helper.gen_range(1..999999), rng_helper.gen_range(1..6));
      (Box::new(rng), extracted_seed)
    },
    RngType::SplitMix64 => {
      let extracted_seed = extract_seed(seed);
      let rng = SplitMix64::seed_from_u64(extracted_seed);
      (Box::new(rng), extracted_seed)
    },
    RngType::Pcg32 => {
      let extracted_seed = extract_seed(seed);
      let rng = Pcg32::seed_from_u64(extracted_seed);
      (Box::new(rng), extracted_seed)
    },
    RngType::Pcg64 => {
      let extracted_seed = extract_seed(seed);
      let rng = Pcg64::seed_from_u64(extracted_seed);
      (Box::new(rng), extracted_seed)
    },
    _ => {
      let (rng, extracted_seed) = from_seed(seed);
      (Box::new(rng), extracted_seed)
    }
  };
  rng
}

pub fn uni_gen_range_f64(rng_type_opt: &Option<RngType>, to_gen_on: &mut UniRng, low: f64, high: f64) -> f64 {
  let rng_type = rng_type_opt.unwrap_or(RngType::Pcg64Mcg);
  match rng_type {
    RngType::Mt19937 => {
      let val = next_f64(to_gen_on);
      val * (high - low) + low
    },
    _ => {
      to_gen_on.gen_range(low..high)
    }
  }
}

use rust_decimal::{prelude::ToPrimitive, Decimal};
pub fn uni_gen_range_f32(rng_type_opt: &Option<RngType>, to_gen_on: &mut UniRng, low: f64, high: f64) -> f32 {
  let rng_type = rng_type_opt.unwrap_or(RngType::Pcg64Mcg);
  match rng_type {
    RngType::Mt19937 => {
      let val = next_f64(to_gen_on);
      let r1 = val * (high - low) + low;
      let rr1 = Decimal::from_f64_retain(r1).unwrap();
      let r1 = rr1.round_dp(10).to_f32().unwrap();
      return r1;
    },
    _ => {
      to_gen_on.gen_range(low as f32..high as f32)
    }
  }
}
