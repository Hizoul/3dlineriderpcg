use web_sys::{Request, RequestInit, RequestMode, Response, console::log_1};
use wasm_bindgen::{JsCast};
use wasm_bindgen_futures::JsFuture;
use js_sys::{Uint8Array, Math::random};
use web_sys::window;
use js_sys::Reflect;

pub fn get_env_variable(var_name: &str) -> Option<String> {
  let wind = window().expect("Global window");
  let replay_path_js = Reflect::get(&wind, &var_name.into()).expect("Replay Path to be set");
  log_1(&format!("got env variable {} as {:?}", var_name, replay_path_js).into());
  replay_path_js.as_string()
}