# Edit this configuration file to define what should be installed on
# your system.  Help is available in the configuration.nix(5) man page
# and in the NixOS manual (accessible by running ‘nixos-help’).
{ config, pkgs, ... }:
{
  imports =
    [
      # Include the results of the hardware scan.
      ./hardware.nix
      ./modules.nix
    ];

  @NETWORK@

@TIMEZONE@

@LOCALE@

@KEYBOARD@

  # Define a user account. Don't forget to set a password with ‘passwd’.
  users.users."@USERNAME@" = {
    isNormalUser = true;
    description = "@FULLNAME@";
    extraGroups = [ "wheel" "networkmanager" "dialout" ];
  };
  @AUTOLOGIN@

  # Allow unfree packages
  environment.sessionVariables.NIXPKGS_ALLOW_UNFREE = "1";

  @PACKAGES@

  # This value determines the NixOS release from which the default
  # settings for stateful data, like file locations and database versions
  # on your system were taken. It‘s perfectly fine and recommended to leave
  # this value at the release version of the first install of this system.
  # Before changing this value read the documentation for this option
  # (e.g. man configuration.nix or on https://nixos.org/nixos/options.html).
  @STATEVERSION@

  programs.nix-data = {
    enable = true;
    systemconfig = "/etc/nixos/systems/@ARCH@/@HOSTNAME@/default.nix";
    flake = "/etc/nixos/flake.nix";
    flakearg = "@HOSTNAME@";
  };
}
