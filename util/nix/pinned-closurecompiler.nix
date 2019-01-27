pkgs@{ lib, fetchurl, jre, ... }:
{ version, sha256 }:

lib.overrideDerivation pkgs.closurecompiler (orig: {
        name = "closure-compiler-${version}";
        inherit version;
        src = fetchurl {
            url = "https://dl.google.com/closure-compiler/compiler-${version}.tar.gz";
            inherit sha256;
        };

        installPhase = ''
            mkdir -p $out/share/java $out/bin
            cp compiler.jar $out/share/java/closure-compiler-v${version}.jar
            makeWrapper ${jre}/bin/java $out/bin/closure-compiler \
              --add-flags "-jar $out/share/java/closure-compiler-v${version}.jar"
        '';
    })
