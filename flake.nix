{
  description = "Hackshell";

  inputs = { nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable"; };

  outputs = { self, nixpkgs }:
    let pkgs = nixpkgs.legacyPackages.x86_64-linux;
    in {

      packages.x86_64-linux.default = pkgs.rustPlatform.buildRustPackage {
        pname = "hackshell";
        version = "0.3.16";
        src = ./.;
        cargoLock.lockFile = ./Cargo.lock;
      };

      devShells.x86_64-linux.default = pkgs.mkShell {
        strictDeps = true;
        nativeBuildInputs = with pkgs; [ pkg-config rustup stdenv.cc ];

        CARGO_BUILD_TARGET = "x86_64-unknown-linux-gnu";

        shellHook = ''
          rustup default stable
          rustup target add x86_64-unknown-linux-gnu
          rustup component add rustfmt rust-analyzer rust-src clippy
        '';

      };

    };
}
