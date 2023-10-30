{ options, config, lib, pkgs, ... }:

with lib;
let
  cfg = config.icicle;
  icicle-autostart = pkgs.makeAutostartItem { name = "org.snowflakeos.Icicle"; package = pkgs.internal.icicle; };
in
{
  options.icicle = with types; {
    enable =
      mkEnableOption "Enable Icicle Installer";
    config = mkOption {
      type = path;
      default = "${pkgs.internal.icicle}/etc/icicle";
      description = "Icicle configuration location";
    };
  };

  config = mkIf cfg.enable {
    environment.systemPackages = with pkgs; [
      internal.icicle
      icicle-autostart
    ];
    environment.etc."icicle".source = cfg.config;
  };
}
