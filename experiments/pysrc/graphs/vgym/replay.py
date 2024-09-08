import os
import zlib
import cbor2
from .compressed_list import CompressedList

def load_replay(path):
  with open(path, "rb") as replay_file:
    replay_data = replay_file.read()
    decompressed_data = zlib.decompress(replay_data)
    actual_replay = cbor2.loads(decompressed_data)
    compressed_list = CompressedList.from_deserialized(actual_replay["episodes"])
    actual_replay["episodes"] = compressed_list
    return actual_replay

def save_replay(renv, path, uid, time_needed=0):
  with open(f"{path}/{uid}.tlr", "wb") as replay_file:
    to_serialize = {
      "episodes": renv.minimal_traces.as_dict(),
      "algo": "",
      "env": renv.name,
      "run_type": 1,
      "uid": "to_gen",
      "reuses": None,
      "is_eval_of": None,
      "hyperparams": None,
      "env_config": {},
      "time_needed": time_needed
    }
    serialized_data = cbor2.dumps(to_serialize)
    compressed_data = zlib.compress(serialized_data)
    replay_file.write(compressed_data)
    replay_file.flush()
  print("Saved replay to ", path)