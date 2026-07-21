{
  mkShell,
  appstream-glib,
  cargo,
  clippy,
  desktop-file-utils,
  gdk-pixbuf,
  gettext,
  gnome-desktop,
  gobject-introspection,
  gtk4,
  internal,
  libadwaita,
  libgweather,
  meson,
  ninja,
  openssl,
  pango,
  parted,
  pkg-config,
  polkit,
  rust-analyzer,
  rustc,
  rustfmt,
  rustPlatform,
  vte-gtk4,
  wrapGAppsHook4,
  pkgs,
  ...
}:

mkShell {
  nativeBuildInputs = [
    appstream-glib
    cargo
    clippy
    desktop-file-utils
    gdk-pixbuf
    gettext
    gnome-desktop
    gobject-introspection
    gtk4
    internal.convertyml
    libadwaita
    libgweather
    meson
    ninja
    openssl
    pango
    parted
    pkg-config
    polkit
    rust-analyzer
    rustc
    rustfmt
    rustPlatform.bindgenHook
    vte-gtk4
    wrapGAppsHook4
  ];

  # Set Environment Variables
  RUST_BACKTRACE = "full";
  RUST_SRC_PATH = "${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";
}
