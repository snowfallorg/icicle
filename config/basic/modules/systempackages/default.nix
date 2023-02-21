{ lib, config, ... }:

let
  cfg = config.modules.snowflakeos.systempackages;
in
{
  options.modules.snowflakeos.systempackages = {
    enable = lib.mkEnableOption "System packages managed by snowflakeos";
  };

  config = lib.mkIf cfg.enable {
    imports = [ ./config.nix ];
  };
}
