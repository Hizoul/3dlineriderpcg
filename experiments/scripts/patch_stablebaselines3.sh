sed -i 's/if isinstance(env, gymnasium.Env)/if True/g' /usr/local/lib/python3.10/dist-packages/stable_baselines3/common/vec_env/patch_gym.py 
sed -i 's/if isinstance(env, gymnasium.Env)/if True/g' /workspaces/.env/lib/python3.10/site-packages/stable_baselines3/common/vec_env/patch_gym.py 