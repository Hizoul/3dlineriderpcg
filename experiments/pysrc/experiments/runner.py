
def run_experiments(env_makers, algo_trainers, repetitions = 1):
  for algo_trainer in algo_trainers:
    for env_maker in env_makers:
      # task = Task.init(project_name='Training', task_name='Python')
      for _ in range(0, repetitions):
        algo_trainer(env_maker)
      # task.close()
