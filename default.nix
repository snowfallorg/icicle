{ pkgs ? import <nixpkgs> { }
, lib ? import <nixpkgs/lib>
}:

pkgs.stdenv.mkDerivation rec {
  pname = "icicle";
  version = "0.0.1";

  src = [ ./. ];

  cargoDeps = pkgs.rustPlatform.fetchCargoTarball {
    inherit src;
    name = "${pname}-${version}";
    hash = "sha256-VpEmNhAxH8nitnc3VI+7SX1HMVwPnIXwPrVzOZOCvlk=";
  };

  nativeBuildInputs = with pkgs; [
    appstream-glib
    polkit
    gettext
    desktop-file-utils
    meson
    ninja
    pkg-config
    git
    wrapGAppsHook4
  ] ++ (with pkgs.rustPlatform; [
    cargoSetupHook
    rust.cargo
    rust.rustc
  ]);

  buildInputs = with pkgs; [
    gdk-pixbuf
    glib
    gtk4
    libadwaita
    openssl
    gnome.adwaita-icon-theme
    desktop-file-utils
    gnome-desktop
    libgweather
    vte-gtk4
    parted
    rustPlatform.bindgenHook
  ];
}
