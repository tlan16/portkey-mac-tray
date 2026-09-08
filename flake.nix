{
  description = "Portkey Mac Tray widget fetching Portkey stats";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
  };

  outputs = { self, nixpkgs }:
    let
      systems = [ "aarch64-darwin" "x86_64-darwin" ];
      forAllSystems = nixpkgs.lib.genAttrs systems;
    in
    {
      packages = forAllSystems (system:
        let
          pkgs = import nixpkgs { inherit system; };
        in
        {
          default = pkgs.rustPlatform.buildRustPackage {
            pname = "portkey-mac-tray";
            version = "0.3.0";
            src = ./.;

            cargoLock = {
              lockFile = ./Cargo.lock;
            };

            buildInputs = [
              pkgs.libiconv
            ];
          };
        }
      );

      apps = forAllSystems (system: {
        default = {
          type = "app";
          program = "${self.packages.${system}.default}/bin/portkey-mac-tray";
        };
      });
    };
}
