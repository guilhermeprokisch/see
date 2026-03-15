{
  description = "Nix development environment and build for see";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs =
    {
      self,
      nixpkgs,
      flake-utils,
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs { inherit system; };
        lib = pkgs.lib;

        see = pkgs.rustPlatform.buildRustPackage {
          pname = "see";
          version = "0.9.1";

          src = ./.;
          cargoLock.lockFile = ./Cargo.lock;

          nativeBuildInputs = with pkgs; [
            pkg-config
          ];

          buildInputs =
            with pkgs;
            lib.optionals stdenv.isDarwin [
              libiconv
            ];

          meta = with lib; {
            description = "A cute cat(1)";
            homepage = "https://github.com/guilhermeprokisch/see";
            license = licenses.mit;
            mainProgram = "see";
            platforms = platforms.unix;
          };
        };
      in
      {
        packages = {
          default = see;
          see = see;
          see-cat = see;
        };

        devShells.default = pkgs.mkShell {
          packages = with pkgs; [
            cargo
            cargo-edit
            cargo-watch
            clippy
            nixfmt
            rust-analyzer
            rustc
            rustfmt
          ];

          buildInputs =
            with pkgs;
            lib.optionals stdenv.isDarwin [
              libiconv
            ];

          shellHook = ''
            echo "Development shell ready. Try: cargo test, cargo run -- README.md, nix build"
          '';
        };

        formatter = pkgs.nixfmt;
      }
    );
}
