{ pkgs, lib, config, inputs, ... }:

{
  packages = with pkgs; [
    SDL2
  ];

  languages.rust = {
    enable = true;
    channel = "stable";
    components = [ "rust-src" ];
  };
}
