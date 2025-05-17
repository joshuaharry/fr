FROM ubuntu:24.04

# Version specifications
ENV GH_VERSION=2.72.0 \
    DEBIAN_FRONTEND=noninteractive

# Install system dependencies
RUN apt-get update && apt-get upgrade -y && apt-get install -y \
    fish \
    xdg-utils \
    unzip \
    less \
    lsof \
    curl \
    sudo \
    gpg \
    xz-utils \
    build-essential \
    ripgrep \
    python-is-python3

# Install the GH CLI
RUN case $(uname -m) in \
    x86_64) ARCH=amd64;; \
    aarch64) ARCH=arm64;; \
    *) echo "Unsupported architecture"; exit 1;; \
    esac && \
    curl -fsSLO https://github.com/cli/cli/releases/download/v${GH_VERSION}/gh_${GH_VERSION}_linux_${ARCH}.tar.gz && \
    tar -xvf gh_${GH_VERSION}_linux_${ARCH}.tar.gz && \
    sudo mv gh_${GH_VERSION}_linux_${ARCH}/bin/gh /usr/local/bin/gh && \
    rm -rf gh_${GH_VERSION}_linux_${ARCH}.tar.gz && \
    rm -rf gh_${GH_VERSION}_linux_${ARCH}

# Add user and set up sudo
RUN useradd -m -s /usr/bin/fish user \
    && echo "user ALL=(ALL) NOPASSWD:ALL" > /etc/sudoers.d/user \
    && chmod 0440 /etc/sudoers.d/user

# Switch to user
USER user

WORKDIR /home/user

# Install Rust
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs > install.sh && \
    sh install.sh -y && \
    rm install.sh