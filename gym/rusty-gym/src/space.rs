use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Space { // i64 should be u64 as no negative array sizes or less than 0 actions are possible
  Discrete(i64),
  BoxedWithRange(Vec<i64>, Vec<f64>, Vec<f64>),
  BoxedWithoutRange(Vec<i64>)
}

impl Space {
  pub fn boxed(sizes: Vec<i64>) -> Space {
    Space::BoxedWithoutRange(sizes)
  }
}


impl Default for Space {
  fn default() -> Space {
    Space::Discrete(1)
  }
}

use std::iter::Product;
pub fn space_to_1d_size(space: &Space) -> usize {
  match space {
    Space::Discrete(_) => 1,
    Space::BoxedWithRange(shape, _, _) => {
      let val: i64 = Product::<&i64>::product(shape.iter());
      val as usize
    },
    Space::BoxedWithoutRange(sizes) => {
      let val: i64 = Product::<&i64>::product(sizes.iter());
      val as usize
    }
  }
}
// todo: figure out if needed. currently used in PolicyGradient Pytorch
pub fn space_to_num(space: &Space) -> usize {
  match space {
    Space::Discrete(size) => *size as usize,
    Space::BoxedWithRange(shape, _, _) => {
      let val: i64 = Product::<&i64>::product(shape.iter());
      val as usize
    },
    Space::BoxedWithoutRange(sizes) => {
      let val: i64 = Product::<&i64>::product(sizes.iter());
      val as usize
    }
  }
}