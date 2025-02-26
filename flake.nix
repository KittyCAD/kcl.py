{
  description = "A flake for a project using uv for dependency management with a devShell";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { nixpkgs, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs { inherit system; };
      in {
        devShell = pkgs.mkShell {
          buildInputs = [
            pkgs.cargo
            pkgs.just
            pkgs.openssl
            pkgs.pkg-config
            pkgs.python313
            pkgs.rustc
            pkgs.uv
          ];
        };
      });
}
