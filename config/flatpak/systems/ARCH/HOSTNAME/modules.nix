{ inputs, config, pkgs, lib, system, ... }:
{

@BOOTLOADER@

  services.flatpak.enable = true;
  modules.gnome.removeUtils = true;

}
