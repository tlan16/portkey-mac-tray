{
  description = "My Mac Tray widget fetching Portkey stats";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
  };

  outputs = { self, nixpkgs }:
    let
      # M4 is aarch64-darwin, but we include x86_64-darwin for completeness
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
            pname = "my-mac-tray";
            version = "0.1.0";

            # The source is the current directory
            src = ./.;

            # Use the existing Cargo.lock to resolve dependency versions
            cargoLock = {
              lockFile = ./Cargo.lock;
            };

            # macOS requires specific native frameworks to compile UI and Network code
            buildInputs = with pkgs.darwin.apple_sdk.frameworks; [
              AppKit
              CoreGraphics
              CoreServices
              Foundation
              Security            # Needed by reqwest (native-tls)
              SystemConfiguration # Needed by reqwest
            ] ++ [
              pkgs.libiconv
            ];
          };
        }
      );

      # Allows you to test it locally without installing by running `nix run`
      apps = forAllSystems (system: {
        default = {
          type = "app";
          program = "${self.packages.${system}.default}/bin/my-mac-tray";
        };
      });
    };
}
