{ config, pkgs, system, ... }:
{
  environment.systemPackages = with pkgs.snowflakeos; [
    nix-software-center
    nixos-conf-editor
    snow
    pkgs.git # For rebuiling with flakes
  ];
  programs.nix-data = {
    enable = true;
    # hostconfig = "/etc/nixos/systems/@ARCH@/@HOSTNAME@/default.nix";
    systemconfig = "/etc/nixos/modules/systempackages/config.nix";
    flake = "/etc/nixos/flake.nix";
    flakearg = "@HOSTNAME@";
  };
  snowflakeos.gnome.enable = true;
  snowflakeos.osInfo.enable = true;
}
