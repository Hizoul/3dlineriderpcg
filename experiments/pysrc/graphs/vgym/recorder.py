from gymnasium import Env
import numpy as np
import random
import sys
from .compressed_list import CompressedList
from .replay import save_replay

def generate_id(alphabet ='0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ_-', size=10):
  alphabet_length = len(alphabet)
  new_string = ""
  for i in range(0, size):
    new_string += alphabet[np.random.randint(0, alphabet_length)]
  return new_string

class record(Env):
  def __init__(self, env, name, compress=True, bucket_size=10, result_dir="./recorded_episodes"):
    self.original_env = env
    self.name = name
    self.episodes = []
    self.episode_actions = []
    self.record = True
    self.compress = compress
    self.bucket_size = bucket_size
    self.minimal_traces = CompressedList(self.bucket_size)
    self.env_seed = 924
    self.result_dir = result_dir
    self.timed = False
    self.manage_seeds = not hasattr(env, "get_used_seed")
    self.start = 0
    self.time_needed = -1
    self.action_space = env.action_space
    self.observation_space = env.observation_space
  def step(self, action):
    self.episode_actions.append(action)
    obs, reward, done, info = self.original_env.step(action)
    return obs, reward, done, info
  def stop_recording(self):
    self.record = False
    self.episode_actions.clear()
  def start_recording(self):
    self.record = True
    self.episode_actions.clear()
  def reset(self):
    if self.record and len(self.episode_actions) > 0:
      to_append = (self.env_seed, self.episode_actions[:])
      if self.compress:
        self.minimal_traces.push(to_append)
      else:
        self.episodes.append(to_append)
      self.episode_actions.clear()
    obs = None
    if self.manage_seeds:
      self.env_seed = random.randint(0, sys.maxsize)
      self.original_env.seed(self.env_seed)
      obs = self.original_env.reset()
    else:
      obs = self.original_env.reset()
      self.env_seed = self.original_env.get_used_seed() # TODO: check if this is enough
    if self.timed:
      self.start = 1 # TODO
    return obs
  def use_seed(self, new_seed):
    self.env_seed = new_seed
    self.original_env.seed(new_seed)
    if self.manage_seeds:
      self.original_env.reset()
  def get_used_seed(self):
    return self.env_seed
  def seed(self, new_seed):
    self.use_seed(new_seed)
  def get_config(self):
    pass
  def load_config(self, new_config):
    pass
  def training_finished(self, time_needed=0):
    self.minimal_traces.finalize()
    uid = generate_id(size=21)
    self.time_needed = time_needed
    save_replay(self, self.result_dir, uid, time_needed=time_needed)
