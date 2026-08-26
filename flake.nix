{
  description = "wam - A simple CLI/TUI web-app manager";

  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

  outputs =
    { nixpkgs, ... }:
    let
      systems = [
        "x86_64-linux"
        "aarch64-linux"
        "x86_64-darwin"
        "aarch64-darwin"
      ];
      forAllSystems = f: nixpkgs.lib.genAttrs systems (system: f nixpkgs.legacyPackages.${system});
    in
    {
      packages = forAllSystems (
        pkgs:
        let
          wam = pkgs.callPackage ./packaging/package.nix { };
        in
        {
          inherit wam;
          default = wam;
        }
      );

      devShells = forAllSystems (pkgs: {
        default = pkgs.mkShell {
          buildInputs = with pkgs; [
            cargo
            rustc
            rustfmt
            clippy
          ];
          shellHook = ''
            mkdir -p .git/hooks
            cat > .git/hooks/pre-commit << 'EOF'
#!/usr/bin/env bash
set -e
echo "Running cargo fmt..."
cargo fmt --check
echo "Running cargo clippy..."
cargo clippy --all-targets -- -D warnings
EOF
            chmod +x .git/hooks/pre-commit
          '';
        };
      });
    };
}
