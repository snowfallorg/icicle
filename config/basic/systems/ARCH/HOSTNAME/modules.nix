{ config, pkgs, system, ... }:
{
  modules.snowflakeos = {
    base.enable = true;
    gnome.enable = true;
    keyboard.enable = true;
    localetime.enable = true;
    systempackages.enable = true;
    users.enable = true;
  };
}
