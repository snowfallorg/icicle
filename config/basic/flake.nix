{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    snowflake.url = "github:snowflakelinux/snowflake-modules";
    nix-data.url = "github:snowflakelinux/nix-data";
    nix-software-center.url = "github:vlinkz/nix-software-center";
    nixos-conf-editor.url = "github:vlinkz/nixos-conf-editor";
    snow.url = "github:snowflakelinux/snow";
  };
  outputs = { self, nixpkgs, ... }@inputs:
    let
      system = "x86_64-linux";
    in
    {
      nixosConfigurations."@HOSTNAME@" = nixpkgs.lib.nixosSystem {
        inherit system;
        modules = [
          ./configuration.nix
          ./snowflake.nix
          inputs.snowflake.nixosModules.snowflake
          inputs.nix-data.nixosModules.${system}.nix-data
        ];
        specialArgs = { inherit inputs; inherit system; };
    };
  };
}
