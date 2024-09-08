FROM ubuntu:22.04
ENV TZ=Europe/Amsterdam
ENV RUSTUP_HOME=/opt/rustup
ENV CARGO_HOME=/opt/cargo
ENV PATH=$PATH:/opt/cargo/bin:/opt/bin
RUN ln -snf /usr/share/zoneinfo/$TZ /etc/localtime && echo $TZ > /etc/timezone
RUN apt-get update && apt-get install -y build-essential python3-dev python-dev python3-pip curl libssl-dev cmake pkg-config libfreetype6 libfreetype6-dev gcc gfortran libblas3 liblapack3 libatlas3-base libatlas-base-dev libblas-dev liblapack-dev m4 python3-venv && apt-get clean
RUN pip3 --no-cache install gym numpy scipy pyyaml matplotlib pandas future kiwisolver pillow
RUN curl https://sh.rustup.rs > install_rustup.sh
RUN sh install_rustup.sh -y
RUN rustup update
RUN cargo install cargo-make cargo-watch
RUN mkdir -p /opt/bin

# Needed for devcontainer vscode
RUN apt-get update && apt-get install -y git && apt-get clean
