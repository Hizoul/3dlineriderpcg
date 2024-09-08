use web_sys::{Request, RequestInit, RequestMode, Response, console::log_1};
use wasm_bindgen::{JsCast};
use wasm_bindgen_futures::JsFuture;
use js_sys::{Uint8Array, Math::random};
use web_sys::window;
use js_sys::Reflect;

pub async fn read_url(url: &str) -> Vec<u8> {
  log_1(&format!("fetching file {:?}", url).into());
  let mut opts = RequestInit::new();
  opts.method("GET");
  opts.mode(RequestMode::Cors);
  
  let request = Request::new_with_str_and_init(&url, &opts).unwrap();
  let window = web_sys::window().unwrap();
  log_1(&format!("about to block on request").into());
  let resp_value = JsFuture::from(window.fetch_with_request(&request)).await.unwrap();
  
  let resp: Response = resp_value.dyn_into().unwrap();
  let data = JsFuture::from(resp.array_buffer().unwrap()).await.unwrap();
  let rs_buf: Uint8Array = Uint8Array::new(&data);
  rs_buf.to_vec()
}

pub async fn read_url_with_post(url: &str) -> Vec<u8> {
  log_1(&format!("fetching file {:?}", url).into());
  let mut opts = RequestInit::new();
  opts.method("POST");
  opts.mode(RequestMode::Cors);
  
  let request = Request::new_with_str_and_init(&url, &opts).unwrap();
  
  let window = web_sys::window().unwrap();
  log_1(&format!("about to block on request").into());
  let resp_value = JsFuture::from(window.fetch_with_request(&request)).await.unwrap();
  
  let resp: Response = resp_value.dyn_into().unwrap();
  let data = JsFuture::from(resp.array_buffer().unwrap()).await.unwrap();
  let rs_buf: Uint8Array = Uint8Array::new(&data);
  rs_buf.to_vec()
}

pub async fn read_url_with_post_data(url: &str, payload: Vec<u8>) -> Vec<u8> {
  use std::collections::HashMap;
  let mut opts = RequestInit::new();
  opts.method("POST");
  opts.mode(RequestMode::Cors);
  let mut data_to_send: HashMap<String, Vec<u8>> = HashMap::new();
  data_to_send.insert("data".to_owned(), payload);
  let serialized = serde_json::to_string(&data_to_send).unwrap();
  opts.body(Some(&serialized.into()));
  
  let request = Request::new_with_str_and_init(&url, &opts).unwrap();
  
  let window = web_sys::window().unwrap();
  let resp_value = JsFuture::from(window.fetch_with_request(&request)).await.unwrap();
  
  let resp: Response = resp_value.dyn_into().unwrap();
  let data = JsFuture::from(resp.array_buffer().unwrap()).await.unwrap();
  let rs_buf: Uint8Array = Uint8Array::new(&data);
  rs_buf.to_vec()
}