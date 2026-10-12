{
  description = "Dev shell for the adsorption-search web app (Rust with the wasm32 target)";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { nixpkgs, fenix, ... }:
    let
      systems = [ "x86_64-linux" "aarch64-darwin" ];

      pkgsFor = system: import nixpkgs {
        inherit system;
        config.allowUnfree = true;
      };

      forAllSystems = nixpkgs.lib.genAttrs systems;
    in
    {
      devShells = forAllSystems (
        system:
        let
          pkgs = pkgsFor system;
          fenixPkgs = fenix.packages.${system};
          # The fenix stable toolchain, following the fenix usage in the
          # parent flake (withComponents on the stable toolchain).
          hostToolchain = (fenixPkgs.stable.withComponents [
            "cargo"
            "clippy"
            "rust-src"
            "rustc"
            "rustfmt"
            "rust-analyzer"
          ]);
          # Merge in the wasm32 target standard library. Fenix exposes target
          # stds as separate packages, not as withComponents entries.
          rustPackages = fenixPkgs.combine [
            hostToolchain
            (fenixPkgs.targets."wasm32-unknown-unknown".stable.rust-std)
          ];
        in
        {
          default = pkgs.mkShell {
            packages = with pkgs; [
              rustPackages
              wasm-pack
            ];
          };
        }
      );
    };
}
