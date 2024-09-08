extern crate serde;

use serde::{de::{DeserializeOwned}, Serialize, Deserialize};
use serde_cbor::{from_slice,to_vec};
use flate2::{Compression, write::{ZlibEncoder, ZlibDecoder}};
use std::io::Write;

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct CompressedVec<T: Serialize + DeserializeOwned + Clone> {
  #[serde(skip)]
  current_decompressed_data: Vec<T>,
  #[serde(skip)]
  iter_decompressed_data: Vec<T>,
  pub max_len_per_bucket: u32,
  compressed_buckets: Vec<Vec<u8>>,
  currently_open_bucket: u32,
  last_compressed_bucket_size: u32
}

impl<T: Serialize + DeserializeOwned + Clone> CompressedVec<T> {
  pub fn new() -> CompressedVec<T> {
    CompressedVec::with_max_len_per_bucket(100)
  }
  pub fn from_list(list: &[T]) -> CompressedVec<T> {
    let mut cv = CompressedVec::new();
    for entry in list {
      cv.push(entry.clone());
    }
    cv
  }
  pub fn with_max_len_per_bucket(max_len_per_bucket: u32) -> CompressedVec<T> {
    let compressed_buckets = vec![];
    CompressedVec {
      current_decompressed_data: Vec::with_capacity(max_len_per_bucket as usize),
      iter_decompressed_data: Vec::with_capacity(max_len_per_bucket as usize),
      max_len_per_bucket,
      compressed_buckets,
      last_compressed_bucket_size: 0,
      currently_open_bucket: u32::MAX-1 // because otherwise ErrorImpl { code: Message("invalid value: integer `18446744073709551615`, expected u32")
    }
  }
  /// Appends an element to the back of a collection.
  ///
  /// # Panics
  ///
  /// Panics if the new capacity exceeds `isize::MAX` bytes.
  ///
  /// # Examples
  ///
  /// ```
  /// use compressed_vec::CompressedVec;
  /// let mut vec: CompressedVec<i32> = CompressedVec::new();
  /// assert_eq!(vec.len(), 0);
  /// vec.push(3);
  /// assert_eq!(vec.len(), 1);
  /// ```
  pub fn push(&mut self, element: T) {
    if self.current_decompressed_data.len() >= (self.max_len_per_bucket as usize) {
      self.last_compressed_bucket_size = self.current_decompressed_data.len() as u32;
      self.compressed_buckets.push(self.compress_bucket(&self.current_decompressed_data));
      self.current_decompressed_data.clear();
    }
    self.current_decompressed_data.push(element);
  }

  pub fn finalize(&mut self) {
    if self.current_decompressed_data.len() > 0 {
      self.last_compressed_bucket_size = self.current_decompressed_data.len() as u32;
      self.compressed_buckets.push(self.compress_bucket(&self.current_decompressed_data));
      self.current_decompressed_data.clear();
    }
  }
  
  pub fn compress_bucket(&self, entries: &Vec<T>) -> Vec<u8> {
    let compressed_data = to_vec(entries).unwrap();
    let mut compressor = ZlibEncoder::new(Vec::with_capacity(compressed_data.len()), Compression::best());
    compressor.write_all(&compressed_data).unwrap();
    compressor.finish().unwrap()
  }

  /// Removes the last element from a vector and returns it, or [`None`] if it
  /// is empty.
  ///
  /// # Examples
  ///
  /// ```
  /// use compressed_vec::CompressedVec;
  /// let mut vec: CompressedVec<i32> = CompressedVec::new();
  /// assert_eq!(vec.len(), 0);
  /// vec.push(3);
  /// vec.push(3);
  /// assert_eq!(vec.len(), 2);
  /// vec.pop();
  /// assert_eq!(vec.len(), 1);
  /// vec.pop();
  /// assert_eq!(vec.len(), 0);
  /// vec.pop();
  /// assert_eq!(vec.len(), 0);
  /// ```
  pub fn pop(&mut self) -> Option<T> {
    let decomp_pop = self.current_decompressed_data.pop();
    if self.current_decompressed_data.is_empty() && !self.compressed_buckets.is_empty() {
      let compressed_bucket_opt = self.compressed_buckets.pop();
      if let Some(compressed_bucket) = compressed_bucket_opt {
        self.current_decompressed_data = CompressedVec::load_bucket(compressed_bucket.as_slice());
        self.last_compressed_bucket_size = if self.compressed_buckets.is_empty() {0} else {self.max_len_per_bucket};
      }
    }
    decomp_pop
  }

  fn load_bucket(compressed_bucket: &[u8]) -> Vec<T> {
    let mut decompressor = ZlibDecoder::new(Vec::with_capacity(compressed_bucket.len()));
    decompressor.write_all(&compressed_bucket).unwrap();
    let decompressed_file_contents = decompressor.finish().unwrap();
    from_slice(decompressed_file_contents.as_slice()).unwrap()
  }

  /// Clears the vector, removing all values.
  ///
  /// Note that this method has no effect on the allocated capacity
  /// of the vector.
  ///
  /// # Examples
  ///
  /// ```
  /// use compressed_vec::CompressedVec;
  /// let mut vec: CompressedVec<i32> = CompressedVec::new();
  /// assert_eq!(vec.len(), 0);
  /// vec.push(3);
  /// assert_eq!(vec.len(), 1);
  /// vec.clear();
  /// assert_eq!(vec.len(), 0);
  /// ```
  pub fn clear(&mut self) {
    self.current_decompressed_data.clear();
    self.compressed_buckets.clear();
    self.currently_open_bucket = u32::MAX-1;
    self.last_compressed_bucket_size = 0;
  }
  /// Returns the number of elements in the vector, also referred to
  /// as its 'length'.
  ///
  /// # Examples
  ///
  /// ```
  /// use compressed_vec::CompressedVec;
  /// let mut vec: CompressedVec<i32> = CompressedVec::new();
  /// assert_eq!(vec.len(), 0);
  /// vec.push(3);
  /// assert_eq!(vec.len(), 1);
  /// ```
  pub fn len(&self) -> usize {
    let multiply_with = if self.compressed_buckets.len() > 0 {self.compressed_buckets.len()-1} else {0};
    (multiply_with * self.max_len_per_bucket as usize) + (self.last_compressed_bucket_size as usize) + self.current_decompressed_data.len()
  }
  /// Returns `true` if the vector contains no elements.
  ///
  /// # Examples
  ///
  /// ```
  /// use compressed_vec::CompressedVec;
  /// let mut vec: CompressedVec<i32> = CompressedVec::new();
  /// assert!(vec.is_empty());
  /// vec.push(3);
  /// assert!(!vec.is_empty());
  /// ```
  pub fn is_empty(&self) -> bool {
    self.len() == 0
  }

  pub fn index(&mut self, idx: usize) -> &T {
    let amount_of_items_in_buckets = self.compressed_buckets.len() as u32 * self.max_len_per_bucket;
    let total_amount_of_items = amount_of_items_in_buckets + self.current_decompressed_data.len() as u32;
    if idx > total_amount_of_items as usize {panic!("Trying to access index {} but only {} are available in CompressedVec", idx, total_amount_of_items)}
    if idx < amount_of_items_in_buckets as usize {
      let required_bucket = idx / (self.max_len_per_bucket as usize);
      if required_bucket != self.currently_open_bucket as usize {
        self.iter_decompressed_data = CompressedVec::load_bucket(&self.compressed_buckets[required_bucket as usize]);
        self.currently_open_bucket = required_bucket as u32;
      }
      &self.iter_decompressed_data[idx % (self.max_len_per_bucket as usize)]
    } else {
      &self.current_decompressed_data[idx % (self.max_len_per_bucket as usize)]
    }
  }

  pub fn clone_to_vec(&mut self) -> Vec<T> {
    let mut vec = Vec::with_capacity(self.len());
    for idx in 0..self.len() {
      vec.push(self.index(idx).clone())
    }
    vec
  }

  pub fn call_after_serialization(&mut self) {
    self.currently_open_bucket = self.compressed_buckets.len() as u32 -1;
    let last_bucket_opt = self.compressed_buckets.pop();
    if let Some(last_bucket) = last_bucket_opt {
      self.current_decompressed_data = CompressedVec::load_bucket(&last_bucket);
      if self.compressed_buckets.len() > 0 {
        let last_bucket:  Vec<T> = CompressedVec::load_bucket(&self.compressed_buckets[self.compressed_buckets.len()-1]);
        self.last_compressed_bucket_size =  last_bucket.len() as u32;
      }
    }
  }
}

#[cfg(test)]
mod tests {
  use super::CompressedVec;
  use serde::{Deserialize, Serialize};
  use rand::{Rng, SeedableRng};
  use rand_pcg::Pcg64Mcg;
  use serde_cbor::{from_slice, to_vec};
  #[derive(Serialize, Deserialize, Clone, PartialEq, Debug, Default)]
  pub struct DummyStruct{
    pub num: f64
  }
  #[test]
  fn functionality_test() {
    let mut rng = Pcg64Mcg::from_entropy();
    let data: Vec<DummyStruct> = (0..101).map(|_| {DummyStruct{num: rng.gen_range(0.0..9999.0)}}).collect();
    let mut cv: CompressedVec<DummyStruct> = CompressedVec::with_max_len_per_bucket(10);
    (0..12).for_each(|i| cv.push(data[i].clone()));
    assert_eq!(cv.len(), 12);
    assert_eq!(cv.compressed_buckets.len(), 1);
    let first_popped = cv.pop();
    assert_eq!(cv.len(), 11);
    assert!(data[11] == first_popped.unwrap());
    assert_eq!(cv.compressed_buckets.len(), 1);
    let second_popped = cv.pop();
    assert!(data[10] == second_popped.unwrap());
    assert_eq!(cv.compressed_buckets.len(), 0);
    assert_eq!(cv.len(), 10);
    cv.push(data[0].clone());
    assert_eq!(cv.compressed_buckets.len(), 1);
    assert_eq!(cv.len(), 11);
    cv.clear();
    assert_eq!(cv.compressed_buckets.len(), 0);
    assert_eq!(cv.len(), 0);
    data.iter().for_each(|item| cv.push(item.clone()));
    assert_eq!(cv.compressed_buckets.len(), 10);
    assert_eq!(cv.len(), 101);
    for i in 0..data.len() {
      assert!(data[i] == cv.index(i).clone());
    }
    let uncompressed_full_vec = cv.clone_to_vec();
    for i in 0..data.len() {
      assert!(data[i] == uncompressed_full_vec[i]);
    }
    cv.finalize();
    let serialized = to_vec(&cv).unwrap();
    let mut reserialized: CompressedVec<DummyStruct> = from_slice(&serialized).unwrap();
    reserialized.call_after_serialization();
    let reserialized_vec = reserialized.clone_to_vec();
    for i in 0..data.len() {
      assert!(data[i] == reserialized.index(i).clone());
      assert!(data[i] == reserialized_vec[i]);
    }

    let mut correct_length: CompressedVec<DummyStruct> = CompressedVec::with_max_len_per_bucket(10);
    assert_eq!(correct_length.len(), 0);
    correct_length.push(data[0].clone());
    assert_eq!(correct_length.len(), 1);
    correct_length.push(data[2].clone());
    assert_eq!(correct_length.len(), 2);
    correct_length.finalize();
    assert_eq!(correct_length.len(), 2);
    let mut correct_length: CompressedVec<DummyStruct> = CompressedVec::with_max_len_per_bucket(10);
    (0..12).for_each(|i| correct_length.push(data[i].clone()));
    assert_eq!(correct_length.len(), 12);
    correct_length.finalize();
    assert_eq!(correct_length.len(), 12);

  }
}
