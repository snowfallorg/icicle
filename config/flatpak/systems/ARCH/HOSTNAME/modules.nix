{ inputs, config, pkgs, lib, system, ... }:
{

@BOOTLOADER@

@DESKTOP@

  environment.gnome.excludePackages = with pkgs.gnome; [
    baobab
    cheese
    eog
    epiphany
    file-roller
    pkgs.gnome-text-editor
    gnome-calculator
    gnome-calendar
    gnome-characters
    gnome-clocks
    # pkgs.gnome-console
    gnome-contacts
    gnome-font-viewer
    gnome-logs
    gnome-maps
    gnome-music
    pkgs.gnome-photos
    # gnome-system-monitor
    gnome-weather
    # nautilus
    pkgs.gnome-connections
    simple-scan
    totem
    # yelp
  ];

}
