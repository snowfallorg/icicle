{ lib, config, ... }:

let
  cfg = config.modules.snowflakeos.base;
in
{
  options.modules.snowflakeos.base = {
    enable = lib.mkEnableOption "SnowflakeOS base packages and configuration";
  };

  config = lib.mkIf cfg.enable {
    imports = [ ./config.nix ];
  };
}
