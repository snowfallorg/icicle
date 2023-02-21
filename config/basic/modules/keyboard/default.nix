{ lib, config, ... }:

let
  cfg = config.modules.snowflakeos.keyboard;
in
{
  options.modules.snowflakeos.keyboard = {
    enable = lib.mkEnableOption "Keyboard related settings";
  };

  config = lib.mkIf cfg.enable {
    imports = [ ./config.nix ];
  };
}
