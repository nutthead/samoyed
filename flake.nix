{
  description = "Samoyed - A modern, minimal, cross-platform Git hooks manager";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };

        lib = pkgs.lib;

        rustToolchain = pkgs.rust-bin.stable.latest.default.override {
          extensions = [
            "rust-src"
            "rustfmt"
            "clippy"
            "llvm-tools-preview"
          ];
        };

        rustPlatform = pkgs.makeRustPlatform {
          cargo = rustToolchain;
          rustc = rustToolchain;
        };

        commonNativeBuildInputs = with pkgs; [
          rustToolchain
          pkg-config
          git
        ];

        devNativeBuildInputs = commonNativeBuildInputs ++ [ pkgs.cargo-tarpaulin ];

        # Build inputs including OpenSSL development libraries
        buildInputs = with pkgs; [
          openssl
          openssl.dev
        ] ++ lib.optionals pkgs.stdenv.isDarwin [
          darwin.apple_sdk.frameworks.Security
          darwin.apple_sdk.frameworks.SystemConfiguration
        ];

        # Environment variables for OpenSSL
        shellVars = {
          OPENSSL_DIR = "${pkgs.openssl.dev}";
          OPENSSL_LIB_DIR = "${pkgs.openssl.out}/lib";
          OPENSSL_INCLUDE_DIR = "${pkgs.openssl.dev}/include";
          PKG_CONFIG_PATH = "${pkgs.openssl.dev}/lib/pkgconfig";
        };

      in
      {
        devShells.default = pkgs.mkShell (shellVars // {
          buildInputs = buildInputs;
          nativeBuildInputs = devNativeBuildInputs;

          shellHook = ''
            echo "🐕 Samoyed development environment"
            echo "Rust toolchain: ${rustToolchain.name} (with rustfmt, clippy)"
            echo "OpenSSL: ${pkgs.openssl.version}"
            echo "cargo-tarpaulin: ${pkgs.cargo-tarpaulin.version}"
            echo ""
            echo "Available commands:"
            echo "  cargo build       - Build the project"
            echo "  cargo test        - Run tests"
            echo "  cargo clippy      - Run linter"
            echo "  cargo fmt         - Format code"
            echo "  cargo tarpaulin   - Generate coverage report"
            echo ""
          '';
        });

        # Package definition for Samoyed
        packages.default = rustPlatform.buildRustPackage {
          pname = "samoyed";
          version = "0.2.0";

          src = ./.;

          cargoLock = {
            lockFile = ./Cargo.lock;
          };

          nativeBuildInputs = commonNativeBuildInputs;
          inherit buildInputs;

          # Set OpenSSL variables for build
          OPENSSL_DIR = "${pkgs.openssl.dev}";
          OPENSSL_LIB_DIR = "${pkgs.openssl.out}/lib";
          OPENSSL_INCLUDE_DIR = "${pkgs.openssl.dev}/include";

          meta = with pkgs.lib; {
            description = "A modern, minimal, cross-platform Git hooks manager";
            homepage = "https://github.com/nutthead/samoyed";
            license = licenses.mit;
            maintainers = [ ];
            platforms = platforms.all;
          };
        };

        # Apps for easy running
        apps.default = flake-utils.lib.mkApp {
          drv = self.packages.${system}.default;
        };
      });
}
