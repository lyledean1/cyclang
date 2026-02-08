FROM ubuntu:23.10

ARG DEBIAN_FRONTEND=noninteractive

RUN apt-get update && apt-get install -y --no-install-recommends \
  g++ \
  llvm-17 \
  llvm-17-tools \
  llvm-17-dev \
  clang-17 \
  libpolly-17-dev \
  libzstd-dev \
  make \
  ninja-build \
  file \
  curl \
  ca-certificates \
  python3 \
  git \
  cmake \
  sudo \
  gdb \
  libedit-dev \
  libssl-dev \
  pkg-config \
  zlib1g-dev \
  xz-utils \
  mingw-w64 \
  libgccjit-13-dev \
  && rm -rf /var/lib/apt/lists/*

RUN update-alternatives --install /usr/bin/clang clang /usr/bin/clang-17 100 && \
    update-alternatives --install /usr/bin/clang++ clang++ /usr/bin/clang++-17 100 && \
    update-alternatives --install /usr/bin/llvm-config llvm-config /usr/bin/llvm-config-17 100

RUN curl https://sh.rustup.rs -sSf | bash -s -- -y

ENV PATH="/root/.cargo/bin:${PATH}"
# Set the LLVM_SYS_17_FFI_WORKAROUND environment variable
ENV LLVM_SYS_17_FFI_WORKAROUND=true

# load cyclang volume
WORKDIR /cyclang

CMD ["/bin/bash"]
