
pub fn build_evaluator<'a, 'b>(result_dir: String) -> RlExperimentHelper {
  let env_params = ReachTargetEnvParams {moving_target: false, ..ReachTargetEnvParams::default()};
  let mut env = build_reach_target_2d_env(Some(env_params));
  let algorithms: Vec<Box<dyn SelfTrainingAlgo>> = vec![];
  let environments: Vec<EnvironmentMaker> = vec![Box::new(|env_config| {
    let mut new_env = build_reach_target_2d_env(None);
    new_env.load_config(env_config);
    Box::new(new_env)
  })];
  RlExperimentHelper::with_config(result_dir, algorithms, environments)
}