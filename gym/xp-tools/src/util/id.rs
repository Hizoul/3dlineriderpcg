pub use nanoid::{format, alphabet::SAFE, rngs::default};

pub fn generate_id() -> String {
  format(default, &SAFE, 21)
}