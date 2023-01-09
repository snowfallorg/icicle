{ config, pkgs, inputs, system, ... }:
{
  environment.systemPackages = [
    inputs.nix-software-center.packages.${system}.nix-software-center
    inputs.nixos-conf-editor.packages.${system}.nixos-conf-editor
    inputs.snow.packages.${system}.snow
    pkgs.git # For rebuiling with github flakes
  ];
  programs.nix-data = {
    systemconfig = "/etc/nixos/configuration.nix";
    flake = "/etc/nixos/flake.nix";
    flakearg = "@HOSTNAME@";
  };
  snowflakeos.gnome.enable = true;
  snowflakeos.osInfo.enable = true;
}
