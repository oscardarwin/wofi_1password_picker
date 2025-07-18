{
  description = "A secure 1Password + wofi Rust launcher";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-25.05";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ rust-overlay.overlays.default ];
        pkgs = import nixpkgs {
          inherit system overlays;
          config.allowUnfree = true;
        };

        rustToolchain = pkgs.rust-bin.stable.latest.default;

        nativeBuildInputs = [
          rustToolchain
          pkgs.pkg-config
        ];

        buildInputs = [
          pkgs.wofi
          pkgs.wl-clipboard
        ];

        wofi-1password = pkgs.rustPlatform.buildRustPackage {
          pname = "wofi-1password";
          version = "0.1.0";

          src = ./.;

          cargoLock = {
            lockFile = ./Cargo.lock;
          };

          inherit nativeBuildInputs buildInputs;

          meta = {
            description = "Wofi-based 1Password credential picker";
            license = pkgs.lib.licenses.mit;
            maintainers = [ ];
          };
        };
      in
      {
        packages.default = wofi-1password;

        devShells.default = pkgs.mkShell {
          inherit nativeBuildInputs buildInputs;
          shellHook = ''
            exec fish
          '';
        };
      }
    );
}
