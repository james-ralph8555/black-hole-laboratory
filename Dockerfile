# Use Amazon Linux 2 as the base image. This is a glibc-based distribution
# compatible with AWS Amplify's requirements.
FROM amazonlinux:2

# Install dependencies required by AWS Amplify's build environment.
# See: https://docs.aws.amazon.com/amplify/latest/userguide/custom-build-image.html
# We also include build tools like gcc and make.
RUN yum update -y && \
    yum install -y \
        bash \
        curl \
        git \
        make \
        openssh-clients \
        tar \
        wget \
        gcc \
        gzip \
        which && \
    yum clean all

# Install Node.js v18 and npm using the official NodeSource repository.
# This is required for frontend builds and tooling. Node.js v22+ requires
# a newer glibc than is available in Amazon Linux 2.
RUN curl -fsSL https://rpm.nodesource.com/setup_18.x | bash - && \
    yum install -y nodejs

# Install Rust using rustup, the standard Rust toolchain installer.
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
# Add cargo to the PATH for subsequent RUN commands and for the container's environment.
ENV PATH="/root/.cargo/bin:${PATH}"

# Install wasm-pack for building Rust-based WebAssembly packages.
RUN curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh

# Mark the workspace directory as safe for git operations. This is necessary
# because the user mounting the volume (from the host) will have a different
# UID than the root user inside the container that runs git.
RUN git config --global --add safe.directory /app

WORKDIR /app

# Set the entrypoint for the container. AWS Amplify's build runner expects
# to be able to run commands using bash.
ENTRYPOINT ["bash", "-c"]
