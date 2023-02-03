{ config, pkgs, inputs, system, ... }:
{
  config = {
    environment.systemPackages = with inputs; [
      nix-software-center.packages.${system}.nix-software-center
      nixos-conf-editor.packages.${system}.nixos-conf-editor
      snow.packages.${system}.snow
      pkgs.git # For rebuiling with github flakes
    ];
    programs.nix-data = {
      systemconfig = "/etc/nixos/systems/@ARCH@/@HOSTNAME@/configuration.nix";
      flake = "/etc/nixos/flake.nix";
      flakearg = "@HOSTNAME@";
    };
    snowflakeos.gnome.enable = true;
    snowflakeos.osInfo.enable = true;
  }
}
