use web_sys::{Request, RequestInit, RequestMode, Response, console::log_1};
use wasm_bindgen::{JsCast};
use wasm_bindgen_futures::JsFuture;
use js_sys::{Uint8Array, Math::random};
use web_sys::window;
use js_sys::Reflect;


pub fn list_dir(dir_name: &str) -> Vec<String> {
  vec!["TODO".to_owned(), "NEEDS".to_owned(), "SOME".to_owned(), "IMPLEMENTATION".to_owned()]
}

pub async fn read_file(url: &str) -> Vec<u8> {
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

pub fn create_dir_if_it_doesnt_exist(dir_name: &str) {
}

pub fn kv_store_get(key: &str) -> String {
  let window = web_sys::window().unwrap();
  let local_storage_opt = window.local_storage().unwrap();
  if let Some(local_storage) = local_storage_opt {
    let item_opt = local_storage.get_item(key).unwrap();
    if let Some(item) = item_opt {
      web_sys::console::log_1(&format!("got item {}", item).into());
      return item;
    }
  }
  String::new()
}

pub fn kv_store_set(key: &str, value: &str) {
  let window = web_sys::window().unwrap();
  let local_storage_opt = window.local_storage().unwrap();
  if let Some(local_storage) = local_storage_opt {
    web_sys::console::log_1(&format!("Seting item").into());
    local_storage.set_item(key, value).unwrap();
    web_sys::console::log_1(&format!("Set item").into());
  }
}