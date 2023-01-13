# Edit this configuration file to define what should be installed on
# your system.  Help is available in the configuration.nix(5) man page
# and in the NixOS manual (accessible by running ‘nixos-help’).
{ config, pkgs, ... }:
@NVIDIAOFFLOAD@
{
  imports =
    [ # Include the results of the hardware scan.
      ./hardware-configuration.nix
    ];

@KERNEL@

@BOOTLOADER@

@NETWORK@

@TIMEZONE@

@LOCALE@

@KEYBOARD@

@DESKTOP@

  # Enable CUPS to print documents.
  services.printing.enable = true;
  # Enable sound with pipewire.
  sound.enable = true;
  hardware.pulseaudio.enable = false;
  security.rtkit.enable = true;
  services.pipewire = {
    enable = true;
    alsa.enable = true;
    alsa.support32Bit = true;
    pulse.enable = true;
  };

  # Define a user account. Don't forget to set a password with ‘passwd’.
  users.users."@USERNAME@" = {
    isNormalUser = true;
    description = "@FULLNAME@";
    extraGroups = [ "wheel" "networkmanager" "dialout" ];
  };
@AUTOLOGIN@
  # Allow unfree packages
  nixpkgs.config.allowUnfree = true;
  environment.sessionVariables.NIXPKGS_ALLOW_UNFREE = "1";

@PACKAGES@

@PACKAGEMANAGERS@

  nix.extraOptions = ''
    experimental-features = nix-command flakes
  '';
  # This value determines the NixOS release from which the default
  # settings for stateful data, like file locations and database versions
  # on your system were taken. It‘s perfectly fine and recommended to leave
  # this value at the release version of the first install of this system.
  # Before changing this value read the documentation for this option
  # (e.g. man configuration.nix or on https://nixos.org/nixos/options.html).
@STATEVERSION@
}
