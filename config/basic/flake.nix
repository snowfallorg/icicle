{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    nix-data = {
      url = "github:snowflakelinux/nix-data";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    snowfall-lib = {
      url = "github:snowfallorg/lib";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    snowflakeos-modules.url = "github:snowflakelinux/snowflakeos-modules";
  };

  outputs = inputs:
    inputs.snowfall-lib.mkFlake {
      inherit inputs;
      src = ./.;

      channels-config.allowUnfree = true;
      systems.modules.nixos = with inputs; [
        nix-data.nixosModules.nix-data
        @BOOTLOADER_MODULE@
        snowflakeos-modules.nixosModules.gnome
        snowflakeos-modules.nixosModules.kernel
        snowflakeos-modules.nixosModules.networking
        snowflakeos-modules.nixosModules.packagemanagers
        snowflakeos-modules.nixosModules.pipewire
        snowflakeos-modules.nixosModules.printing
        snowflakeos-modules.nixosModules.snowflakeos
        snowflakeos-modules.nixosModules.metadata
      ];
    };
}
