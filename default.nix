{ pkgs ? import <nixpkgs> { }
, lib ? import <nixpkgs/lib>
}:
let
  convertyml = (import ./convertyml/default.nix { inherit pkgs; });
in
pkgs.stdenv.mkDerivation rec {
  pname = "icicle";
  version = "0.0.1";

  src = [ ./. ];

  cargoDeps = pkgs.rustPlatform.importCargoLock {
    lockFile = ./Cargo.lock;
    outputHashes = {
      "disk-types-0.1.5" = "sha256-TilJ+6Zgc3g1Vd2vWAwscbLGPIgaBWGb4CUxcTKrvlo=";
      "gnome-desktop-0.4.2" = "sha256-V1KHgbH7uu1l1NJ8XiFVqlkPs/IGazfvINuS39+f8zw=";
      "vte4-0.7.0" = "sha256-BVOKGEQn/VwPhJPPGq1wTcjwcvVraCPPvFE72wNrGB0=";
    };
  };

  nativeBuildInputs = with pkgs; [
    appstream-glib
    convertyml
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
    cargo
    rustc
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
