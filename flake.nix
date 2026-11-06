{
  description = "Cargo subcommand for collecting licenses";

  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";

  outputs = { nixpkgs, ... }:
    let
      systems = [
        "x86_64-linux"
        "aarch64-linux"
        "x86_64-darwin"
        "aarch64-darwin"
      ];
      manifest = builtins.fromTOML (builtins.readFile ./Cargo.toml);
    in
    {
      packages = nixpkgs.lib.genAttrs systems (system:
        let
          pkgs = import nixpkgs { inherit system; };
        in
        rec {
          cargo-licenses = pkgs.rustPlatform.buildRustPackage {
            pname = "cargo-licenses";
            version = manifest.package.version;

            src = pkgs.lib.fileset.toSource {
              root = ./.;
              fileset = pkgs.lib.fileset.unions [
                ./Cargo.toml
                ./Cargo.lock
                ./LICENSE
                ./README.md
                ./src
                ./tests
              ];
            };

            cargoLock.lockFile = ./Cargo.lock;
            cargoTestFlags = [ "--bin" "cargo-licenses" ];

            meta = {
              description = manifest.package.description;
              homepage = manifest.package.repository;
              license = pkgs.lib.licenses.mit;
              mainProgram = "cargo-licenses";
              platforms = systems;
            };
          };

          default = cargo-licenses;
        });
    };
}