{ config, pkgs, system, ... }:
{
  # Define a user account. Don't forget to set a password with ‘passwd’.
  users.users."@USERNAME@" = {
    isNormalUser = true;
    description = "@FULLNAME@";
    extraGroups = [ "wheel" "networkmanager" "dialout" ];
  };
@AUTOLOGIN@
}
