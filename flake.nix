{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs = {
        nixpkgs.follows = "nixpkgs";
        flake-utils.follows = "flake-utils";
      };
    };
  };
  outputs =
    {
      self,
      nixpkgs,
      flake-utils,
      rust-overlay,
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
        rustToolchain = pkgs.pkgsBuildHost.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
        nativeBuildInputs = with pkgs; [
          rustToolchain
          pkg-config
        ];
        buildInputs = with pkgs; [ ];
      in
      with pkgs;
      {
        devShells.default = mkShell {
          inherit buildInputs nativeBuildInputs;
        };
        packages.default =
          let
            manifest = (pkgs.lib.importTOML ./pumpkin/Cargo.toml).package;
          in
          (pkgs.makeRustPlatform {
            rustc = rustToolchain;
            cargo = rustToolchain;
          }).buildRustPackage
            {
              pname = manifest.name;
              version = manifest.version;
              src = ./.;
              cargoLock = {
                lockFile = ./Cargo.lock;
                outputHashes = {
                  "fastnbt-2.5.0" = "sha256-E4WI6SZgkjqUOtbfXfKGfpFH7btEh5V0KpMXSIsuh08=";
                };
              };
            };
      }
    );
}

