{
  # Base code came from Claude (took several prompts).
  description = "Lazy cross-compilation environment for Raspberry Pi 5";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        # Regular aarch64 packages from binary cache
        pkgsArm = import nixpkgs {
          system = "aarch64-linux";
          config = {};
          overlays = [];
        };

        # Cross-compilation setup with lazy loading from binary cache
        pkgsCross = import nixpkgs {
          inherit system;
          crossSystem = {
            config = "aarch64-unknown-linux-gnu";
          };
          overlays = [
            (self: super: {
              # Pull dependencies from aarch64 binary cache instead of cross-compiling
              # Add libraries here as needed, e.g.:
              # inherit (pkgsArm) zlib openssl;
            })
          ];
        };

        #pkgs = import nixpkgs { inherit system; };
        pkgs = nixpkgs.legacyPackages.${system};

      in
      {
        # Development shell with cross-compilation toolchain
        devShells.default = pkgsCross.mkShell {
          # Build-time dependencies (run on host)
          nativeBuildInputs = with pkgsCross.buildPackages; [
            gcc
            gnumake
            pkg-config
            patchelf
          ];

          # Runtime dependencies (run on target, from binary cache)
          buildInputs = with pkgsCross; [
            # Add your C library dependencies here if needed
            # e.g., zlib, openssl, etc.
          ];
        };
      }
    );
}
