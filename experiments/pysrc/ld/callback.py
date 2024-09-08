from stable_baselines3.common.callbacks import BaseCallback


class RecordingCallback(BaseCallback):
    """
    A custom callback that derives from ``BaseCallback``.

    :param verbose: Verbosity level: 0 for no output, 1 for info messages, 2 for debug messages
    """
    def __init__(self, algo_name, eval_of, env_maker, eval_episodes = 100, verbose=0):
        super().__init__(verbose)
        self.algo_name = algo_name
        self.eval_of = eval_of
        self.env_maker = env_maker
        self.rollout_num = 0
        self.eval_episodes = eval_episodes
        # Those variables will be accessible in the callback
        # (they are defined in the base class)
        # The RL model
        # self.model = None  # type: BaseAlgorithm
        # An alias for self.model.get_env(), the environment used for training
        # self.training_env = None  # type: Union[gym.Env, VecEnv, None]
        # Number of time the callback was called
        # self.n_calls = 0  # type: int
        # self.num_timesteps = 0  # type: int
        # local and global variables
        # self.locals = None  # type: Dict[str, Any]
        # self.globals = None  # type: Dict[str, Any]
        # The logger object, used to report things in the terminal
        # self.logger = None  # stable_baselines3.common.logger
        # # Sometimes, for event callback, it is useful
        # # to have access to the parent object
        # self.parent = None  # type: Optional[BaseCallback]

    def _on_training_start(self) -> None:
        """
        This method is called before the first rollout starts.
        """
        print("Start of training")
        pass

    def _on_rollout_start(self) -> None:
        """
        A rollout is the collection of environment interaction
        using the current policy.
        This event is triggered before collecting new samples.
        """
        pass

    def _on_step(self) -> bool:
        """
        This method will be called by the model after each call to `env.step()`.

        For child callback (of an `EventCallback`), this will be called
        when the event is triggered.

        :return: If the callback returns False, training is aborted early.
        """
        return True

    def _on_rollout_end(self) -> None:
        """
        This event is triggered before updating the policy.
        """
        print("End of rollout doing inbetween eval")
        self.rollout_num += 1
        for env in self.training_env.envs:
            cfg = env.get_config()
            run_id = cfg["run_id"]
            self.eval_and_save(self.eval_episodes, f"eval_{self.rollout_num}_{run_id}")
            self.model.save(f"models/eval_{self.rollout_num}_model_{run_id}")

    def eval_and_save(self, episode_amount, path):
        for env in self.training_env.envs:
            eval_env = self.env_maker(path)
            obs = eval_env.reset()
            episode = 0
            while episode < episode_amount:
                action = self.model.predict(obs, deterministic=True)
                action_to_use = action[0]
                state = eval_env.step(action_to_use)
                if state[2] == True:
                    episode += 1
                    obs = eval_env.reset()
                else:
                    obs = state[0]
            for e_env in eval_env.envs:
                e_env.finalize(self.algo_name, self.eval_of)


    def _on_training_end(self) -> None:
        """
        This event is triggered before exiting the `learn()` method.
        """
        print(f"got end of training and {len(self.training_env.envs)}")
        for env in self.training_env.envs:
            env.finalize(self.algo_name, self.eval_of)
            cfg = env.get_config()
            run_id = cfg["run_id"]
            self.eval_and_save(10000, f"eval_final_{run_id}")
            self.model.save(f"models/eval_final_model_{run_id}")