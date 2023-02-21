{ lib, config, ... }:

let
  cfg = config.modules.snowflakeos.gnome;
in
{
  options.modules.snowflakeos.gnome = {
    enable = lib.mkEnableOption "Use the GNOME desktop environment";
  };

  config = lib.mkIf cfg.enable {
    imports = [ ./config.nix ];
  };
}
