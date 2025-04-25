FROM --platform=linux/amd64 ubuntu:latest

RUN apt update
RUN apt install -y curl build-essential iproute2
RUN curl https://sh.rustup.rs -sSf | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"
RUN rustup install stable