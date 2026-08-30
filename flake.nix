{
  description = "Project development environment";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs =
    { nixpkgs, rust-overlay, ... }:
    let
      system = "x86_64-linux";

      pkgs = import nixpkgs {
        inherit system;
        config.allowUnfree = true;
        overlays = [
          rust-overlay.overlays.default
        ];
      };

      rustToolchain = pkgs.rust-bin.stable."1.93.0".default.override {
        extensions = [
          "rustfmt"
          "clippy"
          "rust-analyzer"
        ];

        targets = [
          "wasm32-unknown-unknown"
        ];
      };
    in
    {
      devShells.${system}.default = pkgs.mkShell {
        packages = with pkgs; [
          cocogitto
          cargo-edit
          envsubst
          just
          kubectl
          rustToolchain
          spacetimedb
          nodejs-slim_26
        ];

        shellHook = ''
          echo "🚀 Dev shell"
          echo ""
          just --list
        '';
      };
    };
}
