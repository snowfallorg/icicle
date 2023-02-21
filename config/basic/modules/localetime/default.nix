{ lib, config, ... }:

let
  cfg = config.modules.snowflakeos.localetime;
in
{
  options.modules.snowflakeos.localetime = {
    enable = lib.mkEnableOption "Locale and timezone settings";
  };

  config = lib.mkIf cfg.enable {
    imports = [ ./config.nix ];
  };
}
