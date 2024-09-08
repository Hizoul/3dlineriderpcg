use mt19937::{MT19937};
use xp_tools::rng::extract_seed;
const SIZEOF_INT: i32 = 4;

pub fn hash_seed(input: u128) -> Vec<u32> {
  use sha2::{Sha512, Digest};
  let mut hasher = Sha512::new();
  hasher.update(input.to_string().as_bytes());
  let result = hasher.finalize();
  let to_hash: &[u8] = &result[0..8];
  let padding = SIZEOF_INT - to_hash.len() as i32 % SIZEOF_INT;
  let mut new_bytes = Vec::from(to_hash);
  for _ in 0..padding {new_bytes.push(0);}
  let max_split = new_bytes.len() / SIZEOF_INT as usize;
  let mut seed_ints = Vec::with_capacity(max_split);
  for i in 0..max_split {
    let start = i * SIZEOF_INT as usize;
    let extracted_num = u32::from_le_bytes([new_bytes[start], new_bytes[start+1], new_bytes[start+2], new_bytes[start+3]]);
    if extracted_num > 0 {
      seed_ints.push(extracted_num);
    }
  }
  seed_ints
}

/**
 * Call the returned value with `mt19937::gen_res53` for python equality
 * Port of `np_random` from OpenAI Gym https://github.com/openai/gym/blob/v0.10.5/gym/utils/seeding.py
 **/
pub fn openai_gym_rng(seed: Option<u64>) -> (MT19937, u64) {
  let used_seed = extract_seed(seed);
  let new_seed = hash_seed(used_seed as u128);
  (MT19937::new_with_slice_seed(&new_seed), used_seed)
}



#[cfg(feature = "python")]
#[cfg(test)]
pub mod test {
  use mt19937::{gen_res53 as next_f64};
  use pyo3::prelude::*;
  use pyo3::types::{IntoPyDict};
  use rand_pcg::Pcg64Mcg;
  use rand::{SeedableRng, RngCore};
  use super::openai_gym_rng;
  use crate::util::rng::{uni_gen_range_f64, uni_gen_range_f32, RngType, UniRng};
  const SEED_TO_COMPARE: u64 = 424242;

  fn compare_seed(seed: u64) -> Result<(), PyErr> {
    pyo3::prepare_freethreaded_python();
    Python::with_gil(|py| {
      let gym_seeding_module = py.import("gym.utils.seeding")?;
      let np_random_function = gym_seeding_module.getattr("np_random")?;
      let eval_dict = [("np_random", np_random_function)].into_py_dict(py);
      let gym_generated_number = py.eval(&format!("np_random({})[0].rand()", seed), Some(eval_dict), None)?;
      let (mut pure_rust_rng, _) = openai_gym_rng(Some(seed));
      let rust_next_f64 = next_f64(&mut pure_rust_rng);
      let python_next_f64: f64 = gym_generated_number.extract()?;
      assert_eq!(rust_next_f64, python_next_f64);
      Ok(())
    })
  }

  // #[test]
  // fn rng_equality_test_fixed_seed() -> Result<(), PyErr> {
  //   compare_seed(SEED_TO_COMPARE)
  // }
  // #[test]
  // fn rng_equality_test_random_seed() -> Result<(), PyErr> {
  //   let mut rng = Pcg64Mcg::from_entropy();
  //   compare_seed(rng.next_u64())
  // }
  // #[test]
  // fn rng_equality_high_level() -> Result<(), PyErr> {
  //   pyo3::prepare_freethreaded_python();
  //   Python::with_gil(|py| {
  //     let seed = SEED_TO_COMPARE;
  //     let rng_type = RngType::Mt19937;
  //     let gym_seeding_module = py.import("gym.utils.seeding")?;
  //     let np_random_function = gym_seeding_module.getattr("np_random")?;
  //     let eval_dict = [("np_random", np_random_function)].into_py_dict(py);
  //     let gym_generated_number = py.eval(&format!("np_random({})[0].uniform(low=-0.05, high=0.05, size=(1,))[0]", seed), Some(eval_dict), None)?;
  //     let (pure_rust_rng, _) = openai_gym_rng(Some(seed));
  //     let mut boxed_rng: UniRng = Box::new(pure_rust_rng);
  //     let rust_range = uni_gen_range_f64(&Some(rng_type), &mut boxed_rng, -0.05, 0.05);
  //     let python_range: f64 = gym_generated_number.extract()?;
  //     assert_eq!(rust_range, python_range);


  //     let seed = 894;
  //     let (pure_rust_rng, _) = openai_gym_rng(Some(seed));
  //     let mut boxed_rng: UniRng = Box::new(pure_rust_rng);
  //     let r1 = uni_gen_range_f64(&Some(rng_type), &mut boxed_rng, -0.05, 0.05);
  //     let r2 = uni_gen_range_f64(&Some(rng_type), &mut boxed_rng, -0.05, 0.05);
  //     let r3 = uni_gen_range_f64(&Some(rng_type), &mut boxed_rng, -0.05, 0.05);
  //     let r4 = uni_gen_range_f64(&Some(rng_type), &mut boxed_rng, -0.05, 0.05);
  //     let gym_seeding_module = py.import("gym.utils.seeding")?;
  //     let np_random_function = gym_seeding_module.getattr("np_random")?;
  //     let eval_dict = [("np_random", np_random_function)].into_py_dict(py);
  //     let pr1 = py.eval(&format!("np_random({})[0].uniform(low=-0.05, high=0.05, size=(4,))[0]", seed), Some(eval_dict), None)?;
  //     let p1: f64 = pr1.extract()?;
  //     let pr2 = py.eval(&format!("np_random({})[0].uniform(low=-0.05, high=0.05, size=(4,))[1]", seed), Some(eval_dict), None)?;
  //     let p2: f64 = pr2.extract()?;
  //     let pr3 = py.eval(&format!("np_random({})[0].uniform(low=-0.05, high=0.05, size=(4,))[2]", seed), Some(eval_dict), None)?;
  //     let p3: f64 = pr3.extract()?;
  //     let pr4 = py.eval(&format!("np_random({})[0].uniform(low=-0.05, high=0.05, size=(4,))[3]", seed), Some(eval_dict), None)?;
  //     let p4: f64 = pr4.extract()?;
  //     assert_eq!(r1, p1);
  //     assert_eq!(r2, p2);
  //     assert_eq!(r3, p3);
  //     assert_eq!(r4, p4);


  //     let (pure_rust_rng, _) = openai_gym_rng(Some(seed));
  //     let mut boxed_rng: UniRng = Box::new(pure_rust_rng);
  //     let r1 = uni_gen_range_f32(&Some(rng_type), &mut boxed_rng, -0.05, 0.05);
  //     let r2 = uni_gen_range_f32(&Some(rng_type), &mut boxed_rng, -0.05, 0.05);
  //     let r3 = uni_gen_range_f32(&Some(rng_type), &mut boxed_rng, -0.05, 0.05);
  //     let r4 = uni_gen_range_f32(&Some(rng_type), &mut boxed_rng, -0.05, 0.05);
  //     let gym_seeding_module = py.import("gym.utils.seeding")?;
  //     let np_random_function = gym_seeding_module.getattr("np_random")?;
  //     let eval_dict = [("np_random", np_random_function)].into_py_dict(py);
  //     let pr1 = py.eval(&format!("np_random({})[0].uniform(low=-0.05, high=0.05, size=(4,))[0]", seed), Some(eval_dict), None)?;
  //     let p1: f32 = pr1.extract()?;
  //     let pr2 = py.eval(&format!("np_random({})[0].uniform(low=-0.05, high=0.05, size=(4,))[1]", seed), Some(eval_dict), None)?;
  //     let p2: f32 = pr2.extract()?;
  //     let pr3 = py.eval(&format!("np_random({})[0].uniform(low=-0.05, high=0.05, size=(4,))[2]", seed), Some(eval_dict), None)?;
  //     let p3: f32 = pr3.extract()?;
  //     let pr4 = py.eval(&format!("np_random({})[0].uniform(low=-0.05, high=0.05, size=(4,))[3]", seed), Some(eval_dict), None)?;
  //     let p4: f32 = pr4.extract()?;
  //     assert_eq!(r1, p1);
  //     assert_eq!(r2, p2);
  //     assert_eq!(r3, p3);
  //     assert_eq!(r4, p4);
  //     Ok(())
  //   })
  // }
}
