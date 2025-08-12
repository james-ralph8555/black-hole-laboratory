# Use the official NixOS Nix image as the base.
# This image provides the `nix` command-line tool.
FROM nixpkgs/nix-unstable:nixos-25.05-x86_64-linux

# Install dependencies required by AWS Amplify's build environment.
# See: https://docs.aws.amazon.com/amplify/latest/userguide/custom-build-image.html
# We use nix-env to install them into the container's environment.
RUN nix-channel --add https://nixos.org/channels/nixpkgs-unstable nixpkgs && \
    nix-channel --update && \
    nix-env -iA \
        nixpkgs.bash \
        nixpkgs.curl \
        nixpkgs.git \
        nixpkgs.gnumake \
        nixpkgs.openssh \
        nixpkgs.wget \
        nixpkgs.gnutar \
        nixpkgs.nodejs_22 \
        nixpkgs.busybox && \
    nix-collect-garbage -d

# Create a symlink for curl in /bin, as some scripts expect it there.
# RUN ln -s /root/.nix-profile/bin/curl /bin/curl
#

# Add the Nix profile's bin directory to the PATH. This makes packages installed
# with `nix-env` (like npm) available in the shell.
ENV PATH /root/.nix-profile/bin:$PATH

# Mark the workspace directory as safe for git operations. This is necessary
# because the user mounting the volume (from the host) will have a different
# UID than the root user inside the container that runs git.
RUN git config --global --add safe.directory /app

WORKDIR /app

# Enable Nix flakes and the new 'nix' command experimental features.
# The project's flake.nix requires these features to set up the
# development and build environment for AWS Amplify.
RUN mkdir -p /etc/nix
RUN echo "experimental-features = nix-command flakes" >> /etc/nix/nix.conf

# Set the entrypoint for the container. AWS Amplify's build runner expects
# to be able to run commands using bash.
ENTRYPOINT ["bash", "-c"]
