{
  description = "A real-time, physically-accurate black hole simulator";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, flake-utils, fenix }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};

        # Rust toolchain with wasm32 target, based on https://jordankaye.dev/posts/rust-wasm-nix/
        rust-toolchain = with fenix.packages.${system};
          combine [
            (stable.withComponents [
              "cargo"
              "clippy"
              "rust-src"
              "rustc"
              "rustfmt"
            ])
            targets.wasm32-unknown-unknown.stable.rust-std
          ];

      in {
        devShells.default = pkgs.mkShell {
          name = "black-hole-simulator-dev-shell";

          packages = with pkgs; [
            # Rust + WASM dependencies
            rust-toolchain
            wasm-pack
            llvmPackages.bintools # For lld linker

            # Web development dependencies
            nodejs_22

            # Editor integration for better DX
            nodePackages.typescript-language-server
            vscode-langservers-extracted
          ];

          # Environment variable to fix linking on NixOS, from https://jordankaye.dev/posts/rust-wasm-nix/
          CARGO_TARGET_WASM32_UNKNOWN_UNKNOWN_LINKER = "lld";
        };
      });
}
