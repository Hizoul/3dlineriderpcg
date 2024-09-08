/**
 * Current time in Nanoseconds
 */
pub fn now() -> u128 {
  use std::time::{SystemTime, UNIX_EPOCH};
  let start = SystemTime::now();
  let since_the_epoch = start.duration_since(UNIX_EPOCH).expect("Time went backwards");
  since_the_epoch.as_nanos()
}