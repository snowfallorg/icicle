{ stdenv
, appstream-glib
, cargo
, desktop-file-utils
, gdk-pixbuf
, gettext
, git
, glib
, gnome
, gnome-desktop
, gtk4
, internal
, libadwaita
, libgweather
, meson
, ninja
, openssl
, parted
, pkg-config
, polkit
, rustc
, rustPlatform
, vte-gtk4
, wrapGAppsHook4
}:
let
  convertyml = internal.convertyml;
in
stdenv.mkDerivation rec {
  pname = "icicle";
  version = "0.0.2";

  src = [ ../.. ];

  cargoDeps = rustPlatform.importCargoLock {
    lockFile = ../../Cargo.lock;
    outputHashes = {
      "disk-types-0.1.5" = "sha256-TilJ+6Zgc3g1Vd2vWAwscbLGPIgaBWGb4CUxcTKrvlo=";
      "vte4-0.7.0" = "sha256-BVOKGEQn/VwPhJPPGq1wTcjwcvVraCPPvFE72wNrGB0=";
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
    gnome.adwaita-icon-theme
    gtk4
    libadwaita
    libgweather
    openssl
    parted
    rustPlatform.bindgenHook
    vte-gtk4
  ];
}
