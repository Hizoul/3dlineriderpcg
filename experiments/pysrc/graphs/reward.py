from .vgym import load_replay
import plotly.express as px
import plotly.graph_objects as go
import pandas as pd
import numpy as np
import scipy.stats as st
from os import listdir
from os.path import isfile, join
import re
import matplotlib as mpl
import plotly.io as pio   
pio.kaleido.scope.mathjax = None

# from https://colorbrewer2.org/#type=qualitative&scheme=Paired&n=12
COLOR_SCHEME = [
   "#1f78b4",  "#33a02c", "#e31a1c", "#ff7f00", "#6a3d9a", "#ffff99", "#b15928", "#a6cee3", "#b2df8a",  "#fb9a99", "#fdbf6f", "#cab2d6"
]
COLOR_SCHEME_RGB = [
  "rgba(31, 12, 180, 1.0)", "rgba(51, 160, 44, 1.0)", "rgba(227, 26, 28, 1.0)","rgba(255, 127, 0, 1.0)","rgba(106, 61, 154, 1.0)","rgba(255, 255, 153, 1.0)","rgba(177, 89, 40, 1.0)","rgba(166, 206, 227, 1.0)","rgba(178, 223, 138, 1.0)","rgba(251, 154, 153, 1.0)","rgba(253, 191, 111, 1.0)","rgba(202, 178, 214, 1.0)"
]
COLOR_SCHEME_RGB_AREA = [
  "rgba(31, 12, 180, 0.25)", "rgba(51, 160, 44, 0.25)", "rgba(227, 26, 28, 0.25)","rgba(255, 127, 0, 0.25)","rgba(106, 61, 154, 0.25)","rgba(255, 255, 153, 0.25)","rgba(177, 89, 40, 0.25)","rgba(166, 206, 227, 0.25)","rgba(178, 223, 138, 0.25)","rgba(251, 154, 153, 0.25)","rgba(253, 191, 111, 0.25)","rgba(202, 178, 214, 0.25)"
]

def create_colors(amount):
  color_maker = mpl.colormaps["cubehelix"].resampled(amount + 1)
  colors = []
  area_colors = []
  for i in range(1, amount+1):
    created_color = color_maker(i)
    colors.append(f"rgba({created_color[0]*256}, {created_color[1]*256}, {created_color[2]*256}, 1.0)")
    area_colors.append(f"rgba({created_color[0]*256}, {created_color[1]*256}, {created_color[2]*256}, 0.25)")
  return (colors, area_colors)

def x_episode_ticks(amount):
  x_ticks = []
  for i in range(0, amount):
    x_ticks.append(i)
  return x_ticks

def get_rewards(path):
  replay = load_replay(path)
  return replay["reward_per_episode"]


def eval_sort_key_extractor(file_name):
  pattern = re.compile("eval_([0-9]*)_")
  match = pattern.match(file_name)
  return int(match.groups()[0])

def get_task_completed(run):
  y_vals = []
  for i in range(0, run["episodes"].len()):
    episode = run["episodes"].index(i)
    y_vals.append(episode["task_completion"])
  return y_vals

def get_eval_line(parent_dir, run_path):
  replay = load_replay(run_path)
  id = replay["env_config"]["run_id"]
  files = [f for f in listdir(parent_dir) if isfile(join(parent_dir, f))]
  filtered = []
  for file in files:
    if id in file and file.startswith("eval") and file.endswith(".tlrx"):
      filtered.append(file)
  files = sorted(filtered, key=eval_sort_key_extractor)
  eval_line = []
  for file in files:
    nr = load_replay(f"{parent_dir}/{file}")
    y_vals = get_task_completed(nr)
    eval_line.append(np.mean(y_vals))
  df = pd.DataFrame(dict(
    x = x_episode_ticks(len(eval_line)),
    y = eval_line
  ))
  df = df.sort_values(by="x")
  fig = px.line(df, x="x", y="y"
  )
  fig.write_image('reward.pdf')

def config_matches(env_config, config_to_match):
  matched_number = 0
  keys_to_match = config_to_match.keys()
  for key in keys_to_match:
    if env_config[key] == config_to_match[key]:
      matched_number += 1
  return matched_number == len(keys_to_match)

def get_confidence_interval(data):
  interval = st.t.interval(0.95, len(data) - 1, loc=np.mean(data), scale=st.sem(data))
  return interval


def compare_configs_inbetwee_eval(parent_dir, config_name_combos, file_name = "comparison"):
  files = [f for f in listdir(parent_dir) if isfile(join(parent_dir, f))]
  filtered = []
  data_by_name = {}
  for file in files:
    if file.startswith("eval") and not "final" in file and not file.startswith("highlights") and file.endswith(".tlrx"):
      filtered.append(file)
  for file in filtered:
    replay = load_replay(f"{parent_dir}/{file}")
    config_combo = None
    for (config, name, transfer_from) in config_name_combos:
      if config_matches(replay["env_config"], config):
        if transfer_from == False and replay["is_eval_of"] is None or replay["is_eval_of"] == transfer_from:
          config_combo = (config, name)
    if config_combo is not None:
      config, name = config_combo
      if name not in data_by_name:
        data_by_name[name] = {"rewards": [], "task_completed": []}
      
      id = replay["env_config"]["run_id"]
      run_eval_files = [f for f in listdir(parent_dir) if isfile(join(parent_dir, f))]
      filtered_eval_files = []
      for file in run_eval_files:
        if id in file and file.startswith("eval") and file and file.endswith(".tlrx"):
          filtered_eval_files.append(file)
      run_eval_files = sorted(filtered_eval_files, key=eval_sort_key_extractor)

      reward_vals = []
      task_completed_vals = []
      for file in run_eval_files:
        nr = load_replay(f"{parent_dir}/{file}")
        reward_vals.extend(nr["reward_per_episode"])
        task_completed_vals.extend(get_task_completed(nr))
      data_by_name[name]["rewards"].append(reward_vals)
      data_by_name[name]["task_completed"].append(task_completed_vals)
  prepare_and_plot_data_for_compare(data_by_name, plot_eval = True, graph_name=file_name)
  

def compare_configs(parent_dir, config_name_combos, plot_final_eval = False, file_name = "comparison", x_sort = None, bar_err = False):
  files = collect_replay_files(parent_dir, plot_final_eval)
  data_by_name = {}
  print(f"Assessing {len(files)} relevant files")
  best_in_name = {}
  for file in files:
    replay = load_replay(f"{parent_dir}/{file}")
    config_combo = None
    for (config, name, transfer_from) in config_name_combos:
      if name not in best_in_name:
        best_in_name[name] = [-999, -1]
      if config_matches(replay["env_config"], config):
        if transfer_from == False and replay["is_eval_of"] is None or replay["is_eval_of"] == transfer_from:
          config_combo = (config, name)
    if config_combo is not None:
      config, name = config_combo
      if name not in data_by_name:
        data_by_name[name] = {"rewards": [], "task_completed": []}
      data_by_name[name]["rewards"].append(replay["reward_per_episode"])
      completed = get_task_completed(replay)
      data_by_name[name]["task_completed"].append(completed)
      success_rate = np.average(completed)
      if success_rate >  best_in_name[name][0]:
        best_in_name[name][0] = success_rate
        best_in_name[name][1] = file
  print("best by name", best_in_name)
  if bar_err:
    plotbarerr(data_by_name, plot_final_eval = plot_final_eval, graph_name=file_name, x_sort = x_sort)
  else:
    prepare_and_plot_data_for_compare(data_by_name, plot_final_eval = plot_final_eval, graph_name=file_name, x_sort = x_sort)


def plotbarerr(data_by_name, plot_eval = False, plot_final_eval = False, graph_name = "comparison", x_sort = None):
  y_values_task_completed = []
  y_values_task_completed_ci = []
  fig = go.Figure()
  x_axis = ["No Success", "Track", "Ball", "Both"]
  print(f"Done reading in files data starting to get averages and CI")
  for (name, name_data) in data_by_name.items():
    task_completed = []
    ci_task_completed = []
    shortest_x_task = 999999999
    for tk in name_data["task_completed"]:
      if len(tk) < shortest_x_task:
        shortest_x_task =  len(tk)
    for sub in range(0, len(name_data["task_completed"])):
      successes = [0, 0, 0, 0]
      task_completion_arr = name_data["task_completed"][sub]
      for val in task_completion_arr:
        successes[int(val)] += 1
      task_completed.append(successes)
    min_vals = [9999, 9999, 9999, 9999]
    max_vals = [-9999, -9999, -9999, -9999]
    sums = [0, 0, 0, 0]
    total = len(task_completed)
    for run_successes in task_completed:
      for i in range(0, 4):
        sums[i] += run_successes[i]
        if min_vals[i] > run_successes[i]:
          min_vals[i] = run_successes[i]
        if max_vals[i] < run_successes[i]:
          max_vals[i] = run_successes[i]
    avgs = [sums[0]/total, sums[1]/total, sums[2]/total, sums[3]/total]
    min_val_diff = [avgs[0]-min_vals[0],avgs[1]-min_vals[1],avgs[2]-min_vals[2],avgs[3]-min_vals[3]]
    max_val_diff = [max_vals[0]-avgs[0],max_vals[1]-avgs[1],max_vals[2]-avgs[2],max_vals[3]-avgs[3]]
    print("CALCULATED ", avgs, min_vals, max_vals, task_completed)
    fig.add_trace(go.Bar(
      name=name, x=x_axis, y=avgs, textposition="outside", texttemplate="%{y}",
      error_y=dict(
          type='data',
          symmetric=False,
          array=max_val_diff,
          arrayminus=min_val_diff)
      )
    )

  fig.update_xaxes(title_text="Sucess Type")
  if x_sort is not None:
    fig.update_xaxes(categoryorder="array", categoryarray=x_sort)
  fig.update_yaxes(title_text="# of Episodes")
  graph_name_start = graph_name
  if plot_final_eval:
    graph_name_start += "_eval_final"
  fig.write_image(f"{graph_name_start}_errbars.pdf") 

  print(f"Done calculating relevant data beginning to plot")
  #   completions = []
  #   for i in range(0, len(y_values_rewards)):
  #     y_lower = []
  #     y_upper = []
  #     for (lower, upper) in y_values_task_completed_ci[i][0]:
  #       y_lower.append(lower)
  #       y_upper.append(upper)
  #     completions.append((y_values_task_completed[i][1], y_values_task_completed[i][0], y_lower, y_upper))
  #   completion_graph(completions, f"{graph_name_start}_errbars.pdf", x_sort, True)


def prepare_and_plot_data_for_compare(data_by_name, plot_eval = False, plot_final_eval = False, graph_name = "comparison", x_sort = None):
  y_values_rewards = []
  y_values_task_completed = []
  y_values_rewards_ci = []
  y_values_task_completed_ci = []
  print(f"Done reading in files data starting to get averages and CI")
  for (name, name_data) in data_by_name.items():
    avg_rewards = []
    avg_task_completed = []
    ci_rewards = []
    ci_task_completed = []
    # TODO
    # if len(name_data["rewards"]) == 1:
    #   avg_rewards = name_data["rewards"][0]
    #   avg_task_completed = name_data["task_completed"][0]
    # else:
    shortest_x_rewards = 999999999
    shortest_x_task = 999999999
    for rewards in name_data["rewards"]:
      if len(rewards) < shortest_x_rewards:
        shortest_x_rewards =  len(rewards)
    for tk in name_data["task_completed"]:
      if len(tk) < shortest_x_task:
        shortest_x_task =  len(tk)
      print("GOT LENH", len(tk), name)
    for i in range(0, shortest_x_rewards):
      to_avg_rewards = []
      for sub in range(0, len(name_data["rewards"])):
        to_avg_rewards.append(name_data["rewards"][sub][i])
      avg_rewards.append(np.average(to_avg_rewards))
      ci_rewards.append(get_confidence_interval(to_avg_rewards))
    for i in range(0, shortest_x_task):
      to_avg_task = []
      lowest = 99999
      highest = -99999
      for sub in range(0, len(name_data["task_completed"])):
        val = name_data["task_completed"][sub][i]
        if val > highest:
          highest = val
        if val < lowest:
          lowest = val
        to_avg_task.append(val)
      avg_task_completed.append(np.average(to_avg_task))
      interval = get_confidence_interval(to_avg_task)
      if np.isnan(interval[0]):
        interval = (to_avg_task[0], to_avg_task[0])
      ci_task_completed.append(interval)
    y_values_rewards.append((avg_rewards, name))
    y_values_rewards_ci.append((ci_rewards, name))
    y_values_task_completed.append((avg_task_completed, name))
    y_values_task_completed_ci.append((ci_task_completed, name))
  print(f"Done calculating relevant data beginning to plot")
  graph_name_start = graph_name
  if plot_final_eval:
    graph_name_start += "_eval_final"
    completions = []
    for i in range(0, len(y_values_rewards)):
      y_lower = []
      y_upper = []
      for (lower, upper) in y_values_task_completed_ci[i][0]:
        y_lower.append(lower)
        y_upper.append(upper)
      completions.append((y_values_task_completed[i][1], y_values_task_completed[i][0], y_lower, y_upper))
    completion_graph(completions, f"{graph_name_start}.pdf", x_sort)
    completion_graph(completions, f"{graph_name_start}_errbars.pdf", x_sort, True)
  else:
    if plot_eval:
      graph_name_start += "_eval"
    comparison_graph(y_values_rewards, f"{graph_name_start}_reward_0.pdf", 0, y_err=y_values_rewards_ci)
    comparison_graph(y_values_rewards, f"{graph_name_start}_reward_10.pdf", 10, y_err=y_values_rewards_ci)
    comparison_graph(y_values_rewards, f"{graph_name_start}_reward_50.pdf", 50, y_err=y_values_rewards_ci)
    comparison_graph(y_values_rewards, f"{graph_name_start}_reward_100.pdf", 100, y_err=y_values_rewards_ci)
    comparison_graph(y_values_rewards, f"{graph_name_start}_reward_300.pdf", 300, y_err=y_values_rewards_ci)
    comparison_graph(y_values_task_completed, f"{graph_name_start}_task_completed_0.pdf", 0, y_label="Success Score", y_err=y_values_task_completed_ci)
    comparison_graph(y_values_task_completed, f"{graph_name_start}_task_completed_10.pdf", 10, y_label="Success Score", y_err=y_values_task_completed_ci)
    comparison_graph(y_values_task_completed, f"{graph_name_start}_task_completed_50.pdf", 50, y_label="Success Score", y_err=y_values_task_completed_ci)
    comparison_graph(y_values_task_completed, f"{graph_name_start}_task_completed_100.pdf", 100, y_label="Success Score", y_err=y_values_task_completed_ci)
    comparison_graph(y_values_task_completed, f"{graph_name_start}_task_completed_300.pdf", 300, y_label="Success Score", y_err=y_values_task_completed_ci)


def completion_graph(y_values, file_name, x_sort = None, use_err = False):
  fig = go.Figure()
  x_axis = ["No Success", "Track", "Ball", "Both"]
  if use_err:

    for (name, completions, yl, yu) in y_values:
      track = 0
      ball = 0
      both = 0
      no_sucess = 0
      print("GOT COMPLETIONS ", completions[0:5], yl[0:5], yu[0:5], len(yl))
      for completion in completions:
        rounded = int(completion)
        if rounded < 1:
          no_sucess += 1
        elif rounded < 2:
          track += 1
        elif rounded < 3:
          ball += 1
        elif rounded >= 3:
          both += 1
      
      fig.add_trace(go.Bar(name=name, x=x_axis, y=completions, marker_color="rgb(255,0,0)", marker_pattern_shape="x", textposition="outside", texttemplate="%{y}"))
    
    fig.update_xaxes(title_text="Sucess Type")
    if x_sort is not None:
      fig.update_xaxes(categoryorder="array", categoryarray=x_sort)
    fig.update_yaxes(title_text="# of Episodes")
    fig.write_image(file_name)
  else:
    values_track = []
    values_ball = []
    values_both = []
    values_no_sucess = []
    for (name, completions, yl, yu) in y_values:
      track = 0
      ball = 0
      both = 0
      no_sucess = 0
      for completion in completions:
        rounded = int(completion)
        if rounded < 1:
          no_sucess += 1
        elif rounded < 2:
          track += 1
        elif rounded < 3:
          ball += 1
        elif rounded >= 3:
          both += 1
      x_axis.append(name)
      values_track.append(track)
      values_ball.append(ball)
      values_both.append(both)
      values_no_sucess.append(no_sucess)
      print(f"For {name} got no:{no_sucess} track:{track} ball:{ball} both:{both}")
    fig.add_trace(go.Bar(name="No Success", x=x_axis, y=values_no_sucess, marker_color="rgb(255,0,0)", marker_pattern_shape="x", textposition="outside", texttemplate="%{y}"))
    fig.add_trace(go.Bar(name="Track", x=x_axis, y=values_track, marker_color="rgb(0,0,128)", marker_pattern_shape="-", textposition="outside", texttemplate="%{y}"))
    fig.add_trace(go.Bar(name="Ball", x=x_axis, y=values_ball, marker_color="rgb(0,255,0)", marker_pattern_shape="|", textposition="outside", texttemplate="%{y}"))
    fig.add_trace(go.Bar(name="Both", x=x_axis, y=values_both, marker_color="rgb(0,128,0)", marker_pattern_shape="+", textposition="outside", texttemplate="%{y}"))
    fig.update_layout(barmode='stack')
    fig.update_xaxes(title_text="Sucess Type")
    if x_sort is not None:
      fig.update_xaxes(categoryorder="array", categoryarray=x_sort)
    fig.update_yaxes(title_text="# of Episodes")
    fig.write_image(file_name)

def comparison_graph(y_values, file_name, average_window = 100, y_err = None, y_label = "Reward"):
  #linecolors, fillcolors = create_colors(len(y_values)+2)
  linecolors = COLOR_SCHEME_RGB
  fillcolors = COLOR_SCHEME_RGB_AREA
  fig = go.Figure()
  shortest_x = 999999999
  color_index = 0
  if average_window > 0:
    for i in range (0, len(y_values)):
      value_to_use = y_values[i][0]
      value_to_use = moving_average(value_to_use, average_window).tolist()
      y_values[i] = (value_to_use, y_values[i][1])
  for (y_value, name) in y_values:
    if len(y_value) < shortest_x:
      shortest_x =  len(y_value)
  for (rewards, name) in y_values:
    x_values = x_episode_ticks(shortest_x)
    fig.add_trace(go.Scatter(
      x=x_values, y=rewards[:shortest_x],
      line_color=linecolors[color_index],
      name=name,
    ))
    if y_err is not None:
      for (ci_interval, ci_name) in y_err:
        if ci_name == name:
          y_lower = []
          y_upper = []
          for (lower, upper) in ci_interval:
            y_lower.append(lower)
            y_upper.append(upper)

          if average_window > 0:
            y_lower = moving_average(y_lower, average_window).tolist()
            y_upper = moving_average(y_upper, average_window).tolist()
          y_lower = y_lower[:shortest_x]
          y_upper = y_upper[:shortest_x]
          x_rev = x_values[::-1]
          y_lower = y_lower[::-1]
          fig.add_trace(go.Scatter(
            x=x_values+x_rev,
            y=y_upper+y_lower,
            fillcolor=fillcolors[color_index],
            line_color='rgba(255,255,255,0)',
            fill='toself',
            showlegend=False,
            name=name
          ))
    color_index += 1
        
  fig.update_xaxes(title_text="Episode")
  fig.update_yaxes(title_text=y_label,rangemode="tozero", autorange=False, range=[0,4.0])
  fig.update_traces(mode="lines")
  fig.write_image(file_name)



# print("GOT REPLAY", replay)
# episodes = replay["episodes"]
# print("GOT episode", episodes.len(), episodes.as_dict())
# episode_amount = episodes.len()
# for episode_nr in range(0, episode_amount-1):
#   episode = episodes.index(episode_nr)
#   episode_rewards = episode["reward_per_episode"]
#   print("EPISDOE IS", episode["reward_per_episode"])

# https://stackoverflow.com/a/54628145
def moving_average(x, w):
  return np.convolve(x, np.ones(w), 'valid') / w

def compare_plots(paths):
  fig = go.Figure()
  y_values = []
  df = pd.DataFrame([])
  for (path, name) in paths:
    rewards = get_rewards(path)
    y_values.append((rewards, name))
  shortest_x = 999999999
  for i in range (0, len(y_values)):
    y_values[i] = (moving_average(y_values[i][0], 100).tolist(), y_values[i][1])
  for (y_value, name) in y_values:
    if len(y_value) < shortest_x:
      shortest_x =  len(y_value)
  for (rewards, name) in y_values:
    fig.add_trace(go.Scatter(
      x=x_episode_ticks(shortest_x), y=rewards[:shortest_x],
      name=name,
    ))
  fig.update_traces(mode="lines")
  fig.write_image('reward.pdf')

def make_graph(path):
  rewards = get_rewards(path)
  df = pd.DataFrame(dict(
    x = x_episode_ticks(len(rewards)),
    y = rewards
  ))
  df = df.sort_values(by="x")
  fig = px.line(df, x="x", y="y"
  )
  fig.write_image('reward.pdf')


# x_rev = x[::-1]
# fig.add_trace(go.Scatter(
#     x=x+x_rev,
#     y=y1_upper+y1_lower,
#     fill='toself',
#     fillcolor='rgba(0,100,80,0.2)',
#     line_color='rgba(255,255,255,0)',
#     showlegend=False,
#     name='Fair',
# ))
def collect_replay_files(parent_dir, plot_final_eval = False):
  files = [f for f in listdir(parent_dir) if isfile(join(parent_dir, f))]
  filtered = []
  data_by_name = {}
  for file in files:
    if plot_final_eval:
      if file.startswith("eval_final") and not file.startswith("highlights") and file.endswith(".tlrx"):
        filtered.append(file)
    else:
      if not file.startswith("eval") and not file.startswith("highlights") and file.endswith(".tlrx"):
        filtered.append(file)
  return filtered

def compare_behavior(parent_dir, config_name_combos, plot_final_eval = False, file_name = "comparison"):
  files = collect_replay_files(parent_dir, plot_final_eval)
  data_by_name = {}
  print(f"Assessing {len(files)} relevant files")
  for file in files:
    replay = load_replay(f"{parent_dir}/{file}")
    config_combo = None
    for (config, name, transfer_from) in config_name_combos:
      if config_matches(replay["env_config"], config):
        if transfer_from == False and replay["is_eval_of"] is None or replay["is_eval_of"] == transfer_from:
          config_combo = (config, name)
    if config_combo is not None:
      config, name = config_combo
      if name not in data_by_name:
        data_by_name[name] = {"touch": [], "air": [], "speed": [], "length": [], "end_speed": [], "rotation": []}
      air = []
      touch = []
      speed = []
      end_speed = []
      rotation = []
      length = []
      for i in range(0, replay["episodes"].len()):
        episode = replay["episodes"].index(i)
        ai = episode["additional_info"]
        total_time = int(ai["total_time"])
        touch.append(int(ai["time_rider_touched_track"]) / total_time)
        air.append(int(ai["time_rider_airborne"]) / total_time)
        speed.append(float(ai["overall_velocity"]) / total_time)
        end_speed.append(float(ai["velocity_at_end"]) / 34.0) # TODO: end speed max?
        rotation.append(float(ai["overall_rotation"]) / 20.0)
        length.append(len(episode["log"]) / int(replay["env_config"]["step_limit"]))
      data_by_name[name]["touch"].append(touch)
      data_by_name[name]["air"].append(air)
      data_by_name[name]["speed"].append(speed)
      data_by_name[name]["length"].append(length)
      data_by_name[name]["end_speed"].append(end_speed)
      data_by_name[name]["rotation"].append(rotation)
  prepare_and_plot_data_for_behavior(data_by_name, as_bar=False, plot_final_eval = plot_final_eval, graph_name=f"{file_name}_stats.pdf")
  prepare_and_plot_data_for_behavior(data_by_name, as_bar=True, plot_final_eval = plot_final_eval, graph_name=f"{file_name}_stats_bar.pdf")

def prepare_and_plot_data_for_behavior(data_by_name, as_bar = False, plot_final_eval = False, graph_name = "comparison"):
  # linecolors, fillcolors = create_colors(len(y_values))
  fig = go.Figure()
  categories = ["Touch", "Air", "Speed", "End Speed"]#, "Rotation", "Length"]
  if as_bar:
    i = 0
    for name, values in data_by_name.items():
      cat_values = []
      cat_values.append(np.mean(values["touch"]))
      cat_values.append(np.mean(values["air"]))
      cat_values.append(np.mean(values["speed"]))
      cat_values.append(np.mean(values["end_speed"]))
      # cat_values.append(np.mean(values["rotation"]))
      # cat_values.append(np.mean(values["length"]))
      fig.add_trace(go.Bar(name=name, x=categories, y=cat_values, textposition="outside", marker_color=COLOR_SCHEME_RGB[i]))
      i += 1
  else:    
    for name, values in data_by_name.items():
      cat_values = []
      cat_values.append(np.mean(values["touch"]))
      cat_values.append(np.mean(values["air"]))
      cat_values.append(np.mean(values["speed"]))
      cat_values.append(np.mean(values["end_speed"]))
      cat_values.append(np.mean(values["rotation"]))
      cat_values.append(np.mean(values["length"]))
      fig.add_trace(go.Scatterpolar(
            r=cat_values,
            theta=categories,
            fill='toself',
            name=name
      ))

    fig.update_layout(
      polar=dict(
        radialaxis=dict(
          visible=True,
          range=[0, 1]
        )),
      showlegend=True
    )
  fig.write_image(graph_name)