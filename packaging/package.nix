{
  lib,
  rustPlatform,
}:
rustPlatform.buildRustPackage {
  pname = "wam";
  version = "0.1.1";

  src = lib.cleanSource ../.;

  cargoLock.lockFile = ../Cargo.lock;

  meta = {
    description = "A simple CLI/TUI web-app manager";
    homepage = "https://github.com/bardiya01/wam";
    license = lib.licenses.mit;
    mainProgram = "wam";
    platforms = lib.platforms.unix;
  };
}
