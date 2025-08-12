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
        nixpkgs.openssh \
        nixpkgs.wget \
        nixpkgs.gnutar \
        nixpkgs.nodejs_22 \
        nixpkgs.busybox && \
    nix-collect-garbage -d

# Enable Nix flakes and the new 'nix' command experimental features.
# The project's flake.nix requires these features to set up the
# development and build environment for AWS Amplify.
RUN mkdir -p /etc/nix
RUN echo "experimental-features = nix-command flakes" >> /etc/nix/nix.conf
