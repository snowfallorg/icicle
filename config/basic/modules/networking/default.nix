{ lib, config, ... }:

let
  cfg = config.modules.snowflakeos.networking;
in
{
  options.modules.snowflakeos.networking = {
    enable = lib.mkEnableOption "Networking configuration";
  };

  config = lib.mkIf cfg.enable {
    imports = [ ./config.nix ];
  };
}
