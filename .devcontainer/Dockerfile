FROM docker.io/rust:1.83-bullseye

RUN apt-get update && export DEBIAN_FRONTEND=noninteractive \
    && apt-get -y install git zsh

RUN groupadd -g 1000 code && useradd -u 1000 -g 1000 -s /usr/bin/zsh -m -d /workspace vscode
RUN rustup component add rustfmt rust-analyzer
