FROM nvidia/cuda:11.3.1-runtime-ubuntu20.04
ENV TZ=Europe/Amsterdam
ENV RUSTUP_HOME=/opt/rustup
ENV CARGO_HOME=/opt/cargo
ENV PATH=$PATH:/opt/cargo/bin:/opt/bin
RUN ln -snf /usr/share/zoneinfo/$TZ /etc/localtime && echo $TZ > /etc/timezone
RUN apt-get update && apt-get install -y build-essential python3-dev python-dev python3-pip curl libssl-dev cmake pkg-config libfreetype6 libfreetype6-dev gcc gfortran libblas3 liblapack3 libatlas3-base libatlas-base-dev libblas-dev liblapack-dev m4 python3-venv && apt-get clean
RUN pip3 --no-cache install gym[atari] numpy scipy pyyaml matplotlib pandas future kiwisolver pillow
RUN pip3 --no-cache install torch==1.10.2+cu113 torchvision==0.11.3+cu113 torchaudio==0.10.2+cu113 -f https://download.pytorch.org/whl/cu113/torch_stable.html
RUN pip3 --no-cache install stable-baselines3 maturin cbor2 Pympler
RUN pip3 --no-cache install tensorflow atari_py
RUN curl https://sh.rustup.rs > install_rustup.sh
RUN sh install_rustup.sh -y
RUN rustup update
RUN cargo install cargo-make cargo-watch
RUN mkdir -p /opt/bin
RUN mkdir -p /local/mullermft/3results/reward_comparison
# Needed for EGUI
RUN apt-get update && apt-get install -y libxcb-render0-dev libwayland-egl1 libxcb-shape0-dev libxcb-xfixes0-dev libspeechd-dev libxkbcommon-dev libfontconfig1-dev libx11-6 libx11-dev libxi6 x11-apps libx11-xcb1 libgl1-mesa-dev libxrandr-dev && apt-get clean
# Needed for devcontainer vscode
RUN apt-get update && apt-get install -y git && apt-get clean

# Remove when Atari unneded
#ADD ROMS /atari_roms
#RUN apt-get update && apt-get install -y swig && apt-get clean
#RUN pip3 install atari_py Box2D box2d-py
#RUN python3 -m atari_py.import_roms /atari_roms
#RUN pip3 install joblib
# for pettingzoo
#RUN pip3 install pettingzoo[butterfly,classic] supersuit pyglet
#RUN apt-get update && apt-get install -y libgl1-mesa-glx xvfb python-opengl && apt-get clean
#RUN pip3 --no-cache install ale-py==0.7.4
