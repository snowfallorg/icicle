{ lib, config, ... }:

let
  cfg = config.modules.snowflakeos.users;
in
{
  options.modules.snowflakeos.users = {
    enable = lib.mkEnableOption "User configuration";
  };

  config = lib.mkIf cfg.enable {
    imports = [ ./config.nix ];
  };
}
