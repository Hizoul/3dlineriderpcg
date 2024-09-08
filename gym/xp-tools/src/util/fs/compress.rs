
use std::io::Write;
use serde_cbor::{from_slice,to_vec};
use flate2::{Compression, write::ZlibEncoder};
use flate2::write::ZlibDecoder;
use std::path::Path;
use serde::{Serialize, de::DeserializeOwned};
#[cfg(not(target_arch = "wasm32"))]
use std::fs::{read, write};

#[cfg(not(target_arch = "wasm32"))]
pub fn save_cbor_and_flate_to_path<P: AsRef<Path>, S: Serialize>(path: P, run_data: &S) {
  let serialized_data = to_vec(run_data).unwrap();
  let mut compressor = ZlibEncoder::new(Vec::with_capacity(serialized_data.len()), Compression::best());
  compressor.write_all(&serialized_data).unwrap();
  let compressed_data = compressor.finish().unwrap();
  write(path, compressed_data).unwrap();
}
#[cfg(target_arch = "wasm32")]
pub fn save_cbor_and_flate_to_path<P: AsRef<Path>, S: Serialize>(_path: P, _run_data: &S) {
}

#[cfg(not(target_arch = "wasm32"))]
pub fn save_json_to_path<P: AsRef<Path>, S: Serialize>(path: P, run_data: &S) {
  let serialized_data = serde_json::to_vec(run_data).unwrap();
  write(path, serialized_data).unwrap();
}
#[cfg(target_arch = "wasm32")]
pub fn save_json_to_path<P: AsRef<Path>, S: Serialize>(_path: P, _run_data: &S) {
}

#[cfg(not(target_arch = "wasm32"))]
pub fn load_json<P: AsRef<Path>, S: DeserializeOwned>(path: P) -> S {
  let file_contents = read(path).unwrap();
  serde_json::from_slice(&file_contents).unwrap()
}


#[cfg(not(target_arch = "wasm32"))]
pub fn load_cbor_and_flate_file<P: AsRef<Path>, S: DeserializeOwned>(path: P) -> S {
  let compressed_file_contents = read(path).unwrap();
  let mut decompressor = ZlibDecoder::new(Vec::with_capacity(compressed_file_contents.len()));
  decompressor.write_all(&compressed_file_contents).unwrap();
  let decompressed_file_contents = decompressor.finish().unwrap();
  let slice = decompressed_file_contents.as_slice();
  from_slice(slice).unwrap()
}

pub async fn async_load_cbor_and_flate_file<S: DeserializeOwned>(path: &str) -> S {
  let compressed_file_contents = super::read_file(path).await;
  let mut decompressor = ZlibDecoder::new(Vec::with_capacity(compressed_file_contents.len()));
  decompressor.write_all(&compressed_file_contents).unwrap();
  let decompressed_file_contents = decompressor.finish().unwrap();
  let slice = decompressed_file_contents.as_slice();
  from_slice(slice).unwrap()
}

pub fn load_cbor_and_flate_from_vec<S: DeserializeOwned>(data: Vec<u8>) -> S {
  let compressed_file_contents = data;
  let mut decompressor = ZlibDecoder::new(Vec::with_capacity(compressed_file_contents.len()));
  decompressor.write_all(&compressed_file_contents).unwrap();
  let decompressed_file_contents = decompressor.finish().unwrap();
  let slice = decompressed_file_contents.as_slice();
  from_slice(slice).unwrap()
}