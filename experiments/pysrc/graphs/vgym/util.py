def episode_to_reward_list(env, episode_data):
  if hasattr(env, "stop_recording"):
    env.stop_recording()
  rewards = []
  env.use_seed(episode_data[0])
  for action in episode_data[1]:
    step_res = env.step(action)
    rewards.append(step_res[1])
  if hasattr(env, "start_recording"):
    env.start_recording()
  return rewards

def episode_to_trace(env, episode_data):
  if hasattr(env, "stop_recording"):
    env.stop_recording()
  trace = []
  env.use_seed(episode_data[0])
  for action in episode_data[1]:
    step_res = env.step(action)
    trace.append(step_res)
  if hasattr(env, "start_recording"):
    env.start_recording()
  return trace
  
def episode_without_trace(env, episode_data):
  if hasattr(env, "stop_recording"):
    env.stop_recording()
  trace = 1
  env.use_seed(episode_data[0])
  for action in episode_data[1]:
    step_res = env.step(action)
  if hasattr(env, "start_recording"):
    env.start_recording()
  return trace

def resimulate(env, episodes):
  resimulated = []
  for episode in episodes:
    resimulated.append(episode_to_trace(env, episode))
  return resimulated

def resimulate_parallel(make_env, episodes):
  from joblib import Parallel, delayed
  import os
  jobs = os.cpu_count()
  return Parallel(n_jobs=jobs)(delayed(episode_to_trace)(make_env(), episode) for episode in episodes)

def resimulate_empty_parallel(make_env, episodes):
  from joblib import Parallel, delayed
  import os
  jobs = os.cpu_count()
  return Parallel(n_jobs=jobs)(delayed(episode_without_trace)(make_env(), episode) for episode in episodes)