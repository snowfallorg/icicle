{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    nix-data.inputs.nixpkgs.follows = "nixpkgs";
    nix-data.url = "github:snowflakelinux/nix-data/dev";
    nix-software-center.url = "github:vlinkz/nix-software-center";
    nixos-conf-editor.url = "github:vlinkz/nixos-conf-editor";
    snow.url = "github:snowflakelinux/snow";
    snowfall-lib.inputs.nixpkgs.follows = "nixpkgs";
    snowfall-lib.url = "github:snowfallorg/lib/dev";
    snowflake.inputs.nixpkgs.follows = "nixpkgs";
    snowflake.url = "github:snowflakelinux/snowflake-modules";
  };

  outputs = inputs:
    inputs.snowfall-lib.mkFlake {
      inherit inputs;
      src = ./.;

      channels-config.allowUnfree = true;
      systems.modules = with inputs; [
        snowflake.nixosModules.snowflake
        nix-data.nixosModules.nix-data
      ];
    };
}
