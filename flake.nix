{
  description = "Generic rust Dev Environment";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs =
    {
      nixpkgs,
      flake-utils,
      ...
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs {
          inherit system;
          config = {
            allowUnfree = true;
          };
        };
        stdenv = pkgs.clangStdenv;
        pythonPackages = pkgs.python3Packages;
      in
      {
        devShells.default = (pkgs.mkShell.override { inherit stdenv; }) {
          inputsFrom = [ pkgs.postgresql ];
          buildInputs = with pkgs; [
          ];
          venvDir = ".venv";
          nativeBuildInputs = with pkgs; [
            cargo
            rustc
            openssl
            pkg-config
            rust-analyzer
            rustfmt
            protobuf

            pythonPackages.python
            pythonPackages.venvShellHook
          ];
          LIBCLANG_PATH = "${pkgs.llvmPackages.libclang.lib}/lib";
          CLANG = "${stdenv.cc}/bin/clang";
        };
      }
    );
}
