FROM ubuntu:24.04

ARG DEBIAN_FRONTEND=noninteractive

RUN apt-get update && apt-get install -y --no-install-recommends \
  curl \
  ca-certificates \
  gnupg \
  lsb-release \
  && rm -rf /var/lib/apt/lists/*

RUN curl -fsSL https://apt.llvm.org/llvm-snapshot.gpg.key | gpg --dearmor -o /usr/share/keyrings/llvm-archive-keyring.gpg && \
    echo "deb [signed-by=/usr/share/keyrings/llvm-archive-keyring.gpg] https://apt.llvm.org/noble/ llvm-toolchain-noble-21 main" \
    > /etc/apt/sources.list.d/llvm.list

RUN apt-get update && apt-get install -y --no-install-recommends \
  g++ \
  llvm-21 \
  llvm-21-tools \
  llvm-21-dev \
  clang-21 \
  libpolly-21-dev \
  libzstd-dev \
  make \
  ninja-build \
  file \
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

RUN update-alternatives --install /usr/bin/clang clang /usr/bin/clang-21 100 && \
    update-alternatives --install /usr/bin/clang++ clang++ /usr/bin/clang++-21 100 && \
    update-alternatives --install /usr/bin/llvm-config llvm-config /usr/bin/llvm-config-21 100

RUN curl https://sh.rustup.rs -sSf | bash -s -- -y

ENV PATH="/root/.cargo/bin:${PATH}"
# Set the LLVM_SYS_21_FFI_WORKAROUND environment variable
ENV LLVM_SYS_21_FFI_WORKAROUND=true

# load cyclang volume
WORKDIR /cyclang

CMD ["/bin/bash"]
