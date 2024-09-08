use web_sys::{Request, RequestInit, RequestMode, Response, console::log_1};
use wasm_bindgen::{JsCast};
use wasm_bindgen_futures::JsFuture;
use js_sys::{Uint8Array, Math::random};
use web_sys::window;
use js_sys::Reflect;
/**
 * Current time in Nanoseconds
 */
pub fn now() -> u128 {
  let window = web_sys::window().expect("should have a window in this context");
  let performance = window
      .performance()
      .expect("performance should be available");
  performance.now() as u128 * 1000000
}