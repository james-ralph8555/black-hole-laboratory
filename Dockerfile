# Use the official NixOS Nix image as the base.
# This image provides the `nix` command-line tool.
FROM nixos/nix:latest

# Enable Nix flakes and the new 'nix' command experimental features.
# The project's flake.nix requires these features to set up the
# development and build environment for AWS Amplify.
RUN echo "experimental-features = nix-command flakes" >> /etc/nix/nix.conf
# Use the official NixOS Nix image as the base.
# This image provides the `nix` command-line tool.
FROM nixos/nix:latest

# Enable Nix flakes and the new 'nix' command experimental features.
# The project's flake.nix requires these features to set up the
# development and build environment for AWS Amplify.
RUN echo "experimental-features = nix-command flakes" >> /etc/nix/nix.conf
