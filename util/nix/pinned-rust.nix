pkgs@{ stdenv, lib, fetchurl, fetchzip, callPackage, runCommand, ... }:
{ minorVersion, srcHash, bootstrapHashes }:


with builtins;
let platform =
      if stdenv.system == "i686-linux"
      then "i686-unknown-linux-gnu"
      else if stdenv.system == "x86_64-linux"
      then "x86_64-unknown-linux-gnu"
      else if stdenv.system == "aarch64-linux"
      then "aarch64-unknown-linux-gnu"
      else if stdenv.system == "i686-darwin"
      then "i686-apple-darwin"
      else if stdenv.system == "x86_64-darwin"
      then "x86_64-apple-darwin"
      else throw "missing bootstrap url for platform ${stdenv.system}";

    rustNixFile = file: pkgs.path + ("/pkgs/development/compilers/rust/" + file);

    buildVersion = "1.${toString minorVersion}.0";
    bootstrapVersion = "1.${toString (sub minorVersion 1)}.0";

    bootstrap = callPackage (rustNixFile "binaryBuild.nix") {
        version = bootstrapVersion;
        src = fetchurl {
            url = "https://static.rust-lang.org/dist/rust-${bootstrapVersion}-${platform}.tar.gz";
            sha256 = bootstrapHashes."${platform}";
        };
        inherit platform;
        buildRustPackage = null;
        versionType = "bootstrap";
    };

    filterMap = f: l: builtins.filter (x: x != null) (map f l);

    src = fetchurl {
        url = "https://static.rust-lang.org/dist/rustc-${buildVersion}-src.tar.gz";
        sha256 = srcHash;
    };

    rustc = lib.overrideDerivation pkgs.rustc (old: {
        name = "rustc-${buildVersion}";
        version = buildVersion;
        src = src;
        configureFlags = filterMap
            (s:
                # --disable-locked-deps exists only from 1.17 onward
                if s == "--disable-locked-deps" && lessThan minorVersion 17
                then null
                # Replace the old --local-rust-root with our new
                # rustc-bootstrap derivation
                else if lib.hasPrefix "--local-rust-root=" s
                then "--local-rust-root=${bootstrap.rustc}"
                else s) old.configureFlags;
        nativeBuildInputs = map
            (drv:
                # Replace the old rustc-bootstrap dependency with a new one
                if lib.hasPrefix "rustc-bootstrap" drv.name
                then bootstrap.rustc
                else drv) old.nativeBuildInputs;
    });

    rustSrc = runCommand "unpack-rust-src-${buildVersion}" {
        src = src;
    } ''
        tar -xzf ${src}
        mv rustc-${buildVersion}-src $out
    '';

in { inherit rustc rustSrc; }
