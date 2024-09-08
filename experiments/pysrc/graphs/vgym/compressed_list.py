import sys
import cbor2
import zlib

def pythonize(element):
  if hasattr(element, "tolist"):
    element = element.tolist()
    return pythonize(element)
  elif hasattr(element, "item"):
    return element.item()
  elif isinstance(element, dict):
    new_dict = {}
    for (key, value) in element.items():
      new_dict[key] = pythonize(element[key])
    return new_dict
  elif isinstance(element, list) or isinstance(element, tuple):
    new_list = []
    for sub_element in element:
      new_list.append(pythonize(sub_element))
    return tuple(new_list) if isinstance(element, tuple) else new_list
  
  return element
class CompressedList:
  def __init__(self, max_len):
    self.max_len_per_bucket = max_len
    self.current_decompressed_data = []
    self.iter_decompressed_data = []
    self.compressed_buckets = []
    self.last_compressed_bucket_size = 0
    self.currently_open_bucket = sys.maxsize
  def as_dict(self):
    return {
      "max_len_per_bucket": self.max_len_per_bucket,
      "compressed_buckets": self.compressed_buckets,
      "last_compressed_bucket_size": self.last_compressed_bucket_size,
      "currently_open_bucket": self.currently_open_bucket
    }
  def from_deserialized(deserialized_data):
    new_list = CompressedList(deserialized_data["max_len_per_bucket"])
    new_list.load_list(deserialized_data)
    return new_list
  def from_list(max_len, to_copy):
    new_list = CompressedList(max_len)
    new_list.copy_list(to_copy)
    return new_list
  def copy_list(self, to_copy):
    for entry in to_copy:
      self.push(entry)
  def push(self, element):
    if len(self.current_decompressed_data) >= self.max_len_per_bucket:
      self.last_compressed_bucket_size = len(self.current_decompressed_data)
      self.compressed_buckets.append(CompressedList.compress_bucket(self.current_decompressed_data))
      self.current_decompressed_data.clear()
    self.current_decompressed_data.append(pythonize(element))
  def finalize(self):
    if len(self.current_decompressed_data) > 0:
      self.last_compressed_bucket_size = len(self.current_decompressed_data)
      self.compressed_buckets.append(CompressedList.compress_bucket(self.current_decompressed_data))
      self.current_decompressed_data.clear()
  def compress_bucket(entries):
    serialized_data = cbor2.dumps(entries)
    compressed_data = zlib.compress(serialized_data)
    return compressed_data
  def load_bucket(compressed_bucket):
    if isinstance(compressed_bucket, list):
      compressed_bucket = bytes(compressed_bucket)
    decompressed_data = zlib.decompress(compressed_bucket)
    deserialized_data = cbor2.loads(decompressed_data)
    return deserialized_data
  def load_list(self, deserialized_data):
    if len(deserialized_data["compressed_buckets"]) > 0:
      self.max_len_per_bucket = deserialized_data["max_len_per_bucket"]
      self.compressed_buckets = deserialized_data["compressed_buckets"]
      self.last_compressed_bucket_size = deserialized_data["last_compressed_bucket_size"]
      self.currently_open_bucket = deserialized_data["currently_open_bucket"]
      self.call_after_serialization()
  def pop(self):
    popped_element = self.current_decompressed_data.pop()
    if len(self.current_decompressed_data) == 0 and len(self.compressed_buckets) > 0:
      compressed_bucket = self.compressed_buckets.pop()
      if compressed_bucket is not None:
        self.current_decompressed_data = CompressedList.load_bucket(compressed_bucket)
        self.last_compressed_bucket_size = 0 if len(self.compressed_buckets) == 0 else self.max_len_per_bucket
    return popped_element
  def clear(self):
    self.current_decompressed_data.clear()
    self.compressed_buckets.clear()
    self.currently_open_bucket = sys.maxsize
    self.last_compressed_bucket_size = 0
  def len(self):
    multiply_with = len(self.compressed_buckets) - 1 if len(self.compressed_buckets) > 0 else 0
    return (multiply_with * self.max_len_per_bucket) + (self.last_compressed_bucket_size) + len(self.current_decompressed_data)
  def is_empty(self):
    return self.len() == 0
  def index(self, idx):
    amount_of_items_in_buckets = len(self.compressed_buckets) * self.max_len_per_bucket
    total_amount_of_items = amount_of_items_in_buckets + len(self.current_decompressed_data)
    if idx > total_amount_of_items:
      return None
    if idx < amount_of_items_in_buckets:
      required_bucket = idx // self.max_len_per_bucket
      if required_bucket != self.currently_open_bucket:
        self.iter_decompressed_data = CompressedList.load_bucket(self.compressed_buckets[required_bucket])
        self.currently_open_bucket = required_bucket
      return self.iter_decompressed_data[idx % self.max_len_per_bucket]
    return self.current_decompressed_data[idx % self.max_len_per_bucket]
  def call_after_serialization(self):
    last_bucket = self.compressed_buckets.pop()
    if last_bucket is not None:
      to_pass = bytes(last_bucket) if isinstance(last_bucket, list) else last_bucket
      self.current_decompressed_data = CompressedList.load_bucket(to_pass)
      if len(self.compressed_buckets) > 0:
        last_bucket = CompressedList.load_bucket(self.compressed_buckets[len(self.compressed_buckets)-1])
        self.last_compressed_bucket_size = len(last_bucket)
      else:
        self.last_compressed_bucket_size = 0
  def to_list(self):
    decompressed_list = []
    for i in range(0, self.len()):
      decompressed_list.append(self.index(i))
    return decompressed_list