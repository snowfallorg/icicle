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
      "disk-types-0.1.5" = "sha256-jXQanPALSKekae9wxLtH+dIvfIOB7VRUP+JeLBjrIqE=";
      "gnome-desktop-0.4.0" = "sha256-E0ElNlLikDoMB41xsH7M9Cy/RIqQ3spqZzDmg0mhTpE=";
      "gweather-sys-4.0.0" = "sha256-6ORuEXmPW2GD42p4Lr4VLAXM6TfOhdP9glndY8wPkXk=";
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
