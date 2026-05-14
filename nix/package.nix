{
  lib,
  rustPlatform,
}:

rustPlatform.buildRustPackage {
  pname = "webclassic";
  version = "0.1.0";

  src = lib.fileset.toSource {
    root = ../.;
    fileset = lib.fileset.unions [
      ../Cargo.toml
      ../Cargo.lock
      ../components
      ../src
    ];
  };

  cargoLock = {
    lockFile = ../Cargo.lock;
  };

  meta = {
    description = "Classic CGI Web server revived in modern era";
    homepage = "https://github.com/starryreverie/webclassic";
    mainProgram = "selector4nix";
    license = lib.licenses.gpl3Plus;
    maintainers = with lib.maintainers; [ starryreverie ];
    platforms = lib.platforms.unix;
  };
}
