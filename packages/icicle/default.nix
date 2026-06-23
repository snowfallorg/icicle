{
  stdenv,
  appstream-glib,
  cargo,
  desktop-file-utils,
  gdk-pixbuf,
  gettext,
  git,
  glib,
  adwaita-icon-theme,
  gnome-desktop,
  gtk4,
  internal,
  libadwaita,
  libgweather,
  meson,
  ninja,
  openssl,
  parted,
  pkg-config,
  polkit,
  rustc,
  rustPlatform,
  vte-gtk4,
  wrapGAppsHook4,
}:
let
  convertyml = internal.convertyml;
in
stdenv.mkDerivation {
  pname = "icicle";
  version = "0.0.2";

  src = [ ../.. ];

  cargoDeps = rustPlatform.importCargoLock {
    lockFile = ../../Cargo.lock;
    outputHashes = {
      "disk-types-0.1.5" = "sha256-9QImftXVnuTbtXp0SFDxIvrmE3UxVbXMVAhQfiF/z30=";
    };
  };

  nativeBuildInputs = [
    appstream-glib
    cargo
    convertyml
    desktop-file-utils
    gettext
    git
    meson
    ninja
    pkg-config
    polkit
    rustc
    rustPlatform.cargoSetupHook
    wrapGAppsHook4
  ];

  buildInputs = [
    desktop-file-utils
    gdk-pixbuf
    glib
    gnome-desktop
    adwaita-icon-theme
    gtk4
    libadwaita
    libgweather
    openssl
    parted
    rustPlatform.bindgenHook
    vte-gtk4
  ];
}
