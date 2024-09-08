from experiments.runner import run_experiments
from linerider import make_env
from .callback import RecordingCallback
from .constants import *
from stable_baselines3 import PPO, DQN
from stable_baselines3.common.vec_env import DummyVecEnv
import uuid

REWARD_HEURISTIC = f"[{REWARD_MIMIC_STRAIGHT_LINE_HEURISTIC}, {REWARD_END_BUILD_PHASE_IF_TRACK_REACHES_GOAL}, {REWARD_GOAL_REACHED_BY_TRACK}, {REWARD_GOAL_REACHED_BY_BALL}]"

REWARD_REGULAR = f"[{REWARD_DISTANCE_TO_GOAL_IN_SIMULATION}, {REWARD_END_BUILD_PHASE_IF_TRACK_REACHES_GOAL}, {REWARD_DISTANCE_OF_TRACK_TO_GOAL_AT_END}, {REWARD_GOAL_REACHED_BY_TRACK}, {REWARD_GOAL_REACHED_BY_BALL}, {REWARD_SCOLD_PREMATURE_END}]"

REWARD_REGULAR_AND_BOOST = f"[25, {REWARD_DISTANCE_TO_GOAL_IN_SIMULATION}, {REWARD_END_BUILD_PHASE_IF_TRACK_REACHES_GOAL}, {REWARD_DISTANCE_OF_TRACK_TO_GOAL_AT_END}, {REWARD_GOAL_REACHED_BY_TRACK}, {REWARD_GOAL_REACHED_BY_BALL}, {REWARD_SCOLD_PREMATURE_END}]"

REWARD_AIR_HEURISTIC = f"[{REWARD_MIMIC_STRAIGHT_LINE_HEURISTIC}, {REWARD_END_BUILD_PHASE_IF_TRACK_REACHES_GOAL}, {REWARD_GOAL_REACHED_BY_TRACK}, {REWARD_GOAL_REACHED_BY_BALL}, {REWARD_AIR_TIME}]"

REWARD_AIR_REGULAR = f"[{REWARD_DISTANCE_TO_GOAL_IN_SIMULATION}, {REWARD_END_BUILD_PHASE_IF_TRACK_REACHES_GOAL}, {REWARD_DISTANCE_OF_TRACK_TO_GOAL_AT_END}, {REWARD_GOAL_REACHED_BY_TRACK}, {REWARD_GOAL_REACHED_BY_BALL}, {REWARD_SCOLD_PREMATURE_END}, {REWARD_AIR_TIME}]"

REWARD_TOUCH_HEURISTIC = f"[{REWARD_MIMIC_STRAIGHT_LINE_HEURISTIC}, {REWARD_END_BUILD_PHASE_IF_TRACK_REACHES_GOAL}, {REWARD_GOAL_REACHED_BY_TRACK}, {REWARD_GOAL_REACHED_BY_BALL}, {REWARD_TRACK_TOUCHES}]"

REWARD_TOUCH_REGULAR = f"[{REWARD_DISTANCE_TO_GOAL_IN_SIMULATION}, {REWARD_END_BUILD_PHASE_IF_TRACK_REACHES_GOAL}, {REWARD_DISTANCE_OF_TRACK_TO_GOAL_AT_END}, {REWARD_GOAL_REACHED_BY_TRACK}, {REWARD_GOAL_REACHED_BY_BALL}, {REWARD_SCOLD_PREMATURE_END}, {REWARD_TRACK_TOUCHES}]"

REWARD_CHECKPOINT = f"[{REWARD_DISTANCE_TO_GOAL_IN_SIMULATION}, {REWARD_END_BUILD_PHASE_IF_TRACK_REACHES_GOAL}, {REWARD_DISTANCE_OF_TRACK_TO_GOAL_AT_END}, {REWARD_GOAL_REACHED_BY_TRACK}, {REWARD_GOAL_REACHED_BY_BALL}, {REWARD_SCOLD_PREMATURE_END}, {REWARD_REACH_CHECKPOINT}, {REWARD_TRACK_REACH_CHECKPOINT}]"

def base_config(env_size = 10):
  env = make_env()
  default_config = env.get_config()
  del env
  default_config["max_width"] = f"{env_size}"
  default_config["step_limit"] = f"{env_size}"
  return default_config

def obs_sliding(config):
  config["observation_type"] = f"{OBSERVATION_TYPE_SLIDING_WINDOW}"
  return config

def base_changes(config):
  config["action_type"] = f"{ACTION_TYPE_FREE_POINTS_WITH_TP_RELATIVE}"
  config["target_type"] = f"{TARGET_RANDOM_START_AND_END}"
  config["observation_type"] = f"{OBSERVATION_TYPE_GOAL_AND_LAST_POINT}"
  config["skip_collision_check_on_last_x_pieces"] = "9999"
  config["simulation_steps"] = f"{int(((1000/80) * 600))}"
  config["reward_type"] = REWARD_REGULAR
  return config

def use_reward(reward_type):
  def reward_type_changer(config):
    config["reward_type"] = reward_type
    return config
  return reward_type_changer

def config_target_same_height(config):
  config["target_type"] = f"5"
  return config

def new_goal_gen(config):
  config["use_new_goalgen"] = f"true"
  return config

def config_target_above_start(config):
  config["target_type"] = f"4"
  return config

def config_target_with_checkpoint_below(config):
  config["target_type"] = f"7"
  return config

def config_target_with_checkpoint_above(config):
  config["target_type"] = f"8"
  return config

def config_static_action(config):
  config["action_type"] = f"{ACTION_TYPE_STATIC_WITH_BOOST}"
  return config

def config_free_but_no_type(config):
  config["action_type"] = f"{ACTION_TYPE_FREE_POINTS_RELATIVE}"
  return config

counter_for_names = {}

def env_config_maker(config_maker, name):
  def env_maker(id_to_use = None):
    global counter_for_names
    id = None
    if id_to_use is not None:
      id = id_to_use
    elif name is not None:
      if name not in counter_for_names:
        counter_for_names[name] = 0
      else:
        counter_for_names[name] += 1
      counter = counter_for_names[name] 
      id = f"{name}_runnum-{counter}"
    else:
      id = str(uuid.uuid4())
    config = base_config()
    for cm in config_maker:
      config = cm(config)
    env = DummyVecEnv([lambda: make_env(config, id)])
    return env
  return env_maker

def ppo_trainer(env_maker):
  callback = RecordingCallback("PPO", "", env_maker)
  model = PPO("MlpPolicy", env_maker(), verbose=1)
  model.learn(total_timesteps=int(100000), callback = callback)

def loaded_ppo_trainer(model_path, eval_of):
  def loaded_trainer(env_maker):
    callback = RecordingCallback("PPO", eval_of, env_maker)
    model = PPO.load(model_path, env=env_maker(), verbose=1)
    model.learn(total_timesteps=int(10000), callback = callback)
  return loaded_trainer

def dqn_trainer(env_maker):
  callback = RecordingCallback("DQN", "", env_maker)
  model = DQN("MlpPolicy", env_maker(), verbose=1, learning_starts=20)
  model.learn(total_timesteps=int(100), callback = callback)

def size_maker(size_to_use):
  def size_adjuster(config):
    config["step_limit"] = f"{size_to_use}"
    config["max_width"] = f"{size_to_use}"
    config["simulation_steps"] = f"{int((1000 / 80) * (80 * size_to_use))}"
    return config
  return size_adjuster

def print_configs(configs_to_run):
  for make_env in configs_to_run:
    env = make_env()
    config = env.envs[0].get_config()
    id = config["run_id"]
    print(f"rewards: {config['reward_type']}; Target: {config['target_type']}; Size: {config['max_width']}")

def do_size_experiment():
  configs_to_run = []
  for size in [10, 20, 30, 40, 50]:
    configs_to_run.append(env_config_maker([base_changes, use_reward(REWARD_HEURISTIC), size_maker(size)], f"size_rl_{size}"))
  run_experiments(configs_to_run, [ppo_trainer], 5)

def do_track_only_experiment():
  configs_to_run = []
  configs_to_run.append(env_config_maker([base_changes, use_reward(f"[{REWARD_DISTANCE_TO_GOAL_IN_SIMULATION}, {REWARD_END_BUILD_PHASE_IF_TRACK_REACHES_GOAL}, {REWARD_DISTANCE_OF_TRACK_TO_GOAL_AT_END}, {REWARD_GOAL_REACHED_BY_TRACK}, {REWARD_GOAL_REACHED_BY_BALL}, {REWARD_SCOLD_PREMATURE_END}, {REWARD_SHORTEST_TRACK}]")], "shortest"))
  configs_to_run.append(env_config_maker([base_changes, use_reward(f"[{REWARD_DISTANCE_TO_GOAL_IN_SIMULATION}, {REWARD_END_BUILD_PHASE_IF_TRACK_REACHES_GOAL}, {REWARD_DISTANCE_OF_TRACK_TO_GOAL_AT_END}, {REWARD_GOAL_REACHED_BY_TRACK}, {REWARD_GOAL_REACHED_BY_BALL}, {REWARD_SCOLD_PREMATURE_END}, {REWARD_TRACK_TOUCHES}]")], "reward_touch"))
  configs_to_run.append(env_config_maker([base_changes, use_reward(f"[{REWARD_DISTANCE_TO_GOAL_IN_SIMULATION}, {REWARD_END_BUILD_PHASE_IF_TRACK_REACHES_GOAL}, {REWARD_DISTANCE_OF_TRACK_TO_GOAL_AT_END}, {REWARD_GOAL_REACHED_BY_TRACK}, {REWARD_GOAL_REACHED_BY_BALL}, {REWARD_SCOLD_PREMATURE_END}, {REWARD_AIR_TIME}]")], "reward_air"))
  configs_to_run.append(env_config_maker([base_changes, use_reward(f"[{REWARD_DISTANCE_TO_GOAL_IN_SIMULATION}, {REWARD_END_BUILD_PHASE_IF_TRACK_REACHES_GOAL}, {REWARD_DISTANCE_OF_TRACK_TO_GOAL_AT_END}, {REWARD_GOAL_REACHED_BY_TRACK}, {REWARD_GOAL_REACHED_BY_BALL}, {REWARD_SCOLD_PREMATURE_END}, {REWARD_SPEED_TOTAL}, {REWARD_SPEED_AT_END}, {REWARD_FASTEST_GOAL_REACH}]")], "reward_speed"))
  configs_to_run.append(env_config_maker([base_changes, use_reward(f"[{REWARD_DISTANCE_TO_GOAL_IN_SIMULATION}, {REWARD_END_BUILD_PHASE_IF_TRACK_REACHES_GOAL}, {REWARD_DISTANCE_OF_TRACK_TO_GOAL_AT_END}, {REWARD_GOAL_REACHED_BY_TRACK}, {REWARD_GOAL_REACHED_BY_BALL}, {REWARD_SCOLD_PREMATURE_END}, {REWARD_TRACK_TOUCHES}, {REWARD_SPEED_TOTAL}, {REWARD_SPEED_AT_END}, {REWARD_FASTEST_GOAL_REACH}]")], "reward_touch_and_speed"))
  configs_to_run.append(env_config_maker([base_changes, use_reward(f"[{REWARD_DISTANCE_TO_GOAL_IN_SIMULATION}, {REWARD_END_BUILD_PHASE_IF_TRACK_REACHES_GOAL}, {REWARD_DISTANCE_OF_TRACK_TO_GOAL_AT_END}, {REWARD_GOAL_REACHED_BY_TRACK}, {REWARD_GOAL_REACHED_BY_BALL}, {REWARD_SCOLD_PREMATURE_END}, {REWARD_AIR_TIME}, {REWARD_SPEED_TOTAL}, {REWARD_SPEED_AT_END}, {REWARD_FASTEST_GOAL_REACH}]")], "reward_air_and_speed"))
  configs_to_run.append(env_config_maker([base_changes, use_reward(f"[{REWARD_DISTANCE_TO_GOAL_IN_SIMULATION}, {REWARD_END_BUILD_PHASE_IF_TRACK_REACHES_GOAL}, {REWARD_DISTANCE_OF_TRACK_TO_GOAL_AT_END}, {REWARD_GOAL_REACHED_BY_TRACK}, {REWARD_GOAL_REACHED_BY_BALL}, {REWARD_SCOLD_PREMATURE_END}, {REWARD_MOST_ROTATION}]")], "reward_rotation"))
  run_experiments(configs_to_run, [ppo_trainer], 5)
  
def do_track_only_experiment_20():
  configs_to_run = []
  configs_to_run.append(env_config_maker([base_changes, size_maker(20), use_reward(f"[{REWARD_DISTANCE_TO_GOAL_IN_SIMULATION}, {REWARD_END_BUILD_PHASE_IF_TRACK_REACHES_GOAL}, {REWARD_DISTANCE_OF_TRACK_TO_GOAL_AT_END}, {REWARD_GOAL_REACHED_BY_TRACK}, {REWARD_GOAL_REACHED_BY_BALL}, {REWARD_SCOLD_PREMATURE_END}, {REWARD_SHORTEST_TRACK}]")], "shortest_20"))
  configs_to_run.append(env_config_maker([base_changes, size_maker(20), use_reward(f"[{REWARD_DISTANCE_TO_GOAL_IN_SIMULATION}, {REWARD_END_BUILD_PHASE_IF_TRACK_REACHES_GOAL}, {REWARD_DISTANCE_OF_TRACK_TO_GOAL_AT_END}, {REWARD_GOAL_REACHED_BY_TRACK}, {REWARD_GOAL_REACHED_BY_BALL}, {REWARD_SCOLD_PREMATURE_END}, {REWARD_TRACK_TOUCHES}]")], "reward_touch_20"))
  configs_to_run.append(env_config_maker([base_changes, size_maker(20), use_reward(f"[{REWARD_DISTANCE_TO_GOAL_IN_SIMULATION}, {REWARD_END_BUILD_PHASE_IF_TRACK_REACHES_GOAL}, {REWARD_DISTANCE_OF_TRACK_TO_GOAL_AT_END}, {REWARD_GOAL_REACHED_BY_TRACK}, {REWARD_GOAL_REACHED_BY_BALL}, {REWARD_SCOLD_PREMATURE_END}, {REWARD_AIR_TIME}]")], "reward_air_20"))
  configs_to_run.append(env_config_maker([base_changes, size_maker(20), use_reward(f"[{REWARD_DISTANCE_TO_GOAL_IN_SIMULATION}, {REWARD_END_BUILD_PHASE_IF_TRACK_REACHES_GOAL}, {REWARD_DISTANCE_OF_TRACK_TO_GOAL_AT_END}, {REWARD_GOAL_REACHED_BY_TRACK}, {REWARD_GOAL_REACHED_BY_BALL}, {REWARD_SCOLD_PREMATURE_END}, {REWARD_SPEED_TOTAL}, {REWARD_SPEED_AT_END}, {REWARD_FASTEST_GOAL_REACH}]")], "reward_speed_20"))
  configs_to_run.append(env_config_maker([base_changes, size_maker(20), use_reward(f"[{REWARD_DISTANCE_TO_GOAL_IN_SIMULATION}, {REWARD_END_BUILD_PHASE_IF_TRACK_REACHES_GOAL}, {REWARD_DISTANCE_OF_TRACK_TO_GOAL_AT_END}, {REWARD_GOAL_REACHED_BY_TRACK}, {REWARD_GOAL_REACHED_BY_BALL}, {REWARD_SCOLD_PREMATURE_END}, {REWARD_TRACK_TOUCHES}, {REWARD_SPEED_TOTAL}, {REWARD_SPEED_AT_END}, {REWARD_FASTEST_GOAL_REACH}]")], "reward_touch_and_speed_20"))
  configs_to_run.append(env_config_maker([base_changes, size_maker(20), use_reward(f"[{REWARD_DISTANCE_TO_GOAL_IN_SIMULATION}, {REWARD_END_BUILD_PHASE_IF_TRACK_REACHES_GOAL}, {REWARD_DISTANCE_OF_TRACK_TO_GOAL_AT_END}, {REWARD_GOAL_REACHED_BY_TRACK}, {REWARD_GOAL_REACHED_BY_BALL}, {REWARD_SCOLD_PREMATURE_END}, {REWARD_AIR_TIME}, {REWARD_SPEED_TOTAL}, {REWARD_SPEED_AT_END}, {REWARD_FASTEST_GOAL_REACH}]")], "reward_air_and_speed_20"))
  run_experiments(configs_to_run, [ppo_trainer], 5)

def checkpoint_exp():
  configs_to_run = []
  configs_to_run.append(env_config_maker([base_changes, config_target_with_checkpoint_below, use_reward(REWARD_CHECKPOINT)], "checkpoint_below"))
  configs_to_run.append(env_config_maker([base_changes, config_target_with_checkpoint_above, use_reward(REWARD_CHECKPOINT)], "checkpoint_above"))
  configs_to_run.append(env_config_maker([base_changes, size_maker(20), config_target_with_checkpoint_below, use_reward(REWARD_CHECKPOINT)], "checkpoint_below_20"))
  configs_to_run.append(env_config_maker([base_changes, size_maker(20), config_target_with_checkpoint_above, use_reward(REWARD_CHECKPOINT)], "checkpoint_above_20"))
  run_experiments(configs_to_run, [ppo_trainer], 5)


def checkpoint_exp_sliding():
  configs_to_run = []
  configs_to_run.append(env_config_maker([base_changes, obs_sliding, config_target_with_checkpoint_below, use_reward(REWARD_CHECKPOINT)], "checkpoint_below"))
  configs_to_run.append(env_config_maker([base_changes, obs_sliding, config_target_with_checkpoint_above, use_reward(REWARD_CHECKPOINT)], "checkpoint_above"))
  configs_to_run.append(env_config_maker([base_changes, obs_sliding, size_maker(20), config_target_with_checkpoint_below, use_reward(REWARD_CHECKPOINT)], "checkpoint_below_20"))
  configs_to_run.append(env_config_maker([base_changes, obs_sliding, size_maker(20), config_target_with_checkpoint_above, use_reward(REWARD_CHECKPOINT)], "checkpoint_above_20"))
  run_experiments(configs_to_run, [ppo_trainer], 5)

def do_basic_exp():
  regular_top_to_bottom = env_config_maker([base_changes], "top_to_bottom")
  regular_top_to_bottom_no_tp = env_config_maker([base_changes, config_free_but_no_type], "top_to_bottom_no_tp")
  static_regular_top_to_bottom = env_config_maker([base_changes, config_static_action], "top_to_bottom_static")
  mimic_heuristic_top_to_bottom = env_config_maker([base_changes, use_reward(REWARD_HEURISTIC)], "top_to_bottom_mimic")
  # static_mimic_heuristic_top_to_bottom = env_config_maker([base_changes, use_reward(REWARD_HEURISTIC), config_static_action], "top_to_bottom_static")
  regular_same = env_config_maker([base_changes, config_target_same_height, use_reward(REWARD_REGULAR_AND_BOOST)], "same")
  regular_same_no_help = env_config_maker([base_changes, config_target_same_height, use_reward(REWARD_REGULAR)], "same_no_help")
  regular_same_no_tp = env_config_maker([base_changes, config_target_same_height, config_free_but_no_type], "same_no_tp")
  mimic_same = env_config_maker([base_changes, use_reward(REWARD_HEURISTIC), config_target_same_height], "same_mimic")
  regular_above = env_config_maker([base_changes, config_target_above_start, use_reward(REWARD_REGULAR_AND_BOOST)], "above")
  mimic_above = env_config_maker([base_changes, use_reward(REWARD_HEURISTIC), config_target_above_start], "above_mimic")

  # regular_air_top_to_bottom = env_config_maker([base_changes, use_reward(REWARD_AIR_REGULAR)])
  # static_regular_air_top_to_bottom = env_config_maker([base_changes, config_static_action, use_reward(REWARD_AIR_REGULAR)])
  # mimic_heuristic_air_top_to_bottom = env_config_maker([base_changes, use_reward(REWARD_AIR_HEURISTIC)])
  #run_experiments([regular_top_to_bottom, regular_top_to_bottom_no_tp, mimic_heuristic_top_to_bottom], [ppo_trainer], 5)
  #run_experiments([static_regular_top_to_bottom], [ppo_trainer], 5)
  #run_experiments([regular_same, regular_same_no_tp, regular_same_no_help, mimic_same], [ppo_trainer], 5)
  run_experiments([regular_above, mimic_above], [ppo_trainer], 5)
  # run_experiments([regular_air_top_to_bottom, mimic_heuristic_air_top_to_bottom], [ppo_trainer], 3)

def sliding_exp():
  configs_to_run = []
  # configs_to_run.append(env_config_maker([base_changes, obs_sliding], "sliding_top_to_bottom"))
  configs_to_run.append(env_config_maker([base_changes, obs_sliding, config_target_above_start, use_reward(REWARD_REGULAR_AND_BOOST)], "sliding_above"))
  # configs_to_run.append(env_config_maker([base_changes, obs_sliding, size_maker(20)], "sliding_top_to_bottom_20"))
  # configs_to_run.append(env_config_maker([base_changes, obs_sliding, size_maker(30)], "sliding_top_to_bottom_30"))
  # configs_to_run.append(env_config_maker([base_changes, obs_sliding, size_maker(40)], "sliding_top_to_bottom_40"))
  run_experiments(configs_to_run, [ppo_trainer], 5)
 
def sliding_new_goal_exp():
  configs_to_run = []
  #configs_to_run.append(env_config_maker([base_changes, new_goal_gen, obs_sliding], "sliding_top_to_bottom"))
  #configs_to_run.append(env_config_maker([base_changes, new_goal_gen, obs_sliding, config_target_same_height], "sliding_above"))
  configs_to_run.append(env_config_maker([base_changes, new_goal_gen, obs_sliding, config_target_above_start, use_reward(REWARD_REGULAR_AND_BOOST)], "sliding_same"))
  run_experiments(configs_to_run, [ppo_trainer], 5)

  
def transfer_sliding_experiment():
  run_experiments([
    # env_config_maker([base_changes, obs_sliding, size_maker(20)], "sliding_transfer_10_20"),
    # env_config_maker([base_changes, obs_sliding, config_target_same_height, size_maker(20), use_reward(REWARD_REGULAR_AND_BOOST)], "sliding_transfer_10_same_20"),
    env_config_maker([base_changes, obs_sliding, config_target_with_checkpoint_above, use_reward(REWARD_CHECKPOINT)], "sliding_transfer_10_chkabove_10"),
    env_config_maker([base_changes, obs_sliding, config_target_with_checkpoint_below, size_maker(20), use_reward(REWARD_CHECKPOINT)], "sliding_transfer_10_chkabove_20"),
    # env_config_maker([base_changes, obs_sliding, config_target_with_checkpoint_below, use_reward(REWARD_CHECKPOINT)], "sliding_transfer_10_chkbelow_10"),
    # env_config_maker([base_changes, obs_sliding, config_target_with_checkpoint_below, size_maker(20), use_reward(REWARD_CHECKPOINT)], "sliding_transfer_10_chkbelow_20")
  ], [loaded_ppo_trainer("./models_for_transfer/eval_final_model_sliding_top_to_bottom_runnum-3.zip", "stb_complex")], 5)
  # run_experiments([
  #   env_config_maker([base_changes, obs_sliding, size_maker(30)], "sliding_transfer_20_30"),
  #   # env_config_maker([base_changes, obs_sliding, config_target_same_height, size_maker(20), use_reward(REWARD_REGULAR_AND_BOOST)], "sliding_transfer_10_same_20"),
  #   # env_config_maker([base_changes, obs_sliding, config_target_with_checkpoint_below, size_maker(20), use_reward(REWARD_CHECKPOINT)], "sliding_transfer_10_chkbelow_20")
  # ], [loaded_ppo_trainer("./models_for_transfer/eval_final_model_sliding_transfer_10_20_runnum-2.zip", "sliding20")], 5)
  # run_experiments([
  #   env_config_maker([base_changes, obs_sliding, size_maker(10)], "sliding_transfer_20_10"),
  #   env_config_maker([base_changes, obs_sliding, size_maker(30)], "sliding_transfer_20_30"),
  # ], [loaded_ppo_trainer("./models_for_transfer/eval_final_model_sliding_top_to_bottom_20_runnum-3.zip", "sliding20")], 5)

def transfer_experiment():
  run_experiments([
    # env_config_maker([base_changes], "transfer_same_bottom"),
    env_config_maker([base_changes, config_target_above_start], "transfer_same_up"),
    # env_config_maker([base_changes, config_target_with_checkpoint_below, use_reward(REWARD_CHECKPOINT)], "transfer_same_chkbelow")
  ], [loaded_ppo_trainer("./models_for_transfer/eval_final_model_same_runnum-0.zip", "same")], 5)
  # run_experiments([
  #   # env_config_maker([base_changes, config_target_same_height, use_reward(REWARD_REGULAR_AND_BOOST)], "transfer_bottom_same"),
  #   env_config_maker([base_changes, config_target_above_start], "transfer_bottom_up"),
  #   # env_config_maker([base_changes, config_target_with_checkpoint_below, use_reward(REWARD_CHECKPOINT)], "transfer_bottom_chkbelow")
  # ], [loaded_ppo_trainer("./models_for_transfer/eval_final_model_top_to_bottom_runnum-0.zip", "topbot")], 5)
