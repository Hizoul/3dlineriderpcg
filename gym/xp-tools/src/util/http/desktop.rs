
pub async fn read_url(url: &str) -> Vec<u8> {
  let resp = reqwest::get(url).await.unwrap();
  let bytes = resp.bytes().await.unwrap();
  bytes.to_vec()
}

pub async fn read_url_with_post(url: &str) -> Vec<u8> {
  let client = reqwest::Client::new();
  let resp = client.post(url).send().await.unwrap();
  let bytes = resp.bytes().await.unwrap();
  bytes.to_vec()
}

pub async fn read_url_with_post_data(url: &str, data: Vec<u8>) -> Vec<u8> {
  use std::collections::HashMap;
  use serde_json::to_string;
  let client = reqwest::Client::new();
  let mut data_to_send: HashMap<String, Vec<u8>> = HashMap::new();
  data_to_send.insert("data".to_owned(), data);
  let resp = client.post(url).body(to_string(&data_to_send).unwrap()).send().await.unwrap();
  let bytes = resp.bytes().await.unwrap();
  bytes.to_vec()
}