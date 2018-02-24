pkgs@{ stdenv, lib, callPackage, fetchFromGitHub, python3Packages, ... }:
{ version, sha256 }:

let imagemagick = lib.overrideDerivation pkgs.imagemagick (_: {
        name = "imagemagick-${version}";
        inherit version;
        src = fetchFromGitHub {
            owner = "ImageMagick";
            repo = "ImageMagick";
            rev = version;
            inherit sha256;
        };
    });

    Wand = python3Packages.Wand.override { inherit imagemagick; };

/*
    wandFile = file: pkgs.path + "/pkgs/development/python-modules/Wand";
    Wand = callPackage wandFile {
        inherit imagemagick;
    };
*/

in { inherit imagemagick Wand; }

