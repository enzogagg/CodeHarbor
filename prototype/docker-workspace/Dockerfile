FROM ubuntu:24.04

ENV DEBIAN_FRONTEND=noninteractive

RUN apt-get update && apt-get install -y \
    build-essential \
    ca-certificates \
    clang \
    clang-format \
    cmake \
    curl \
    file \
    g++ \
    gcc \
    gdb \
    gcovr \
    git \
    gnupg \
    lcov \
    nano \
    openssh-client \
    pkg-config \
    python3 \
    python3-pip \
    python3-venv \
    strace \
    sudo \
    tree \
    unzip \
    valgrind \
    wget \
    && rm -rf /var/lib/apt/lists/*

RUN useradd \
    --create-home \
    --shell /bin/bash \
    --groups sudo \
    dev \
    && echo "dev ALL=(ALL) NOPASSWD:ALL" \
    > /etc/sudoers.d/dev

RUN curl -fsSL https://code-server.dev/install.sh | sh

RUN mkdir -p \
    /workspace \
    /home/dev/.config/code-server \
    /home/dev/.local/share/code-server \
    /command-history \
    && chown -R dev:dev \
    /workspace \
    /home/dev \
    /command-history

USER dev
WORKDIR /workspace

EXPOSE 8080 3000 5173 8000

CMD ["code-server", \
     "--bind-addr", "0.0.0.0:8080", \
     "--auth", "password", \
     "/workspace"]
