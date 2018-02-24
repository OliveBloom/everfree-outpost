pkgs@{ lib, callPackage, runCommand, rustc, ... }:


with builtins;
let buildRustCratePath = pkgs.path + "/pkgs/build-support/rust/build-rust-crate.nix";
    buildRustCrate = callPackage buildRustCratePath { inherit rustc; };

    rustDep = nv: s: 
        let parsed = parseDrvName nv;
        in buildRustCrate (s // {
            crateName = parsed.name;
            version = parsed.version;
        });

    crateName = drv: lib.strings.replaceStrings ["-"] ["_"] drv.libName;

    buildRustLibDir = drvs:
        let genCmd = drv: "ln -sv " +
                "${drv}/lib/lib${crateName drv}-${drv.metadata}.rlib " +
                "$out/lib${crateName drv}.rlib\n";
        in runCommand "rust-lib-dir" { buildInputs = drvs; }
            ("mkdir $out\n" + lib.concatMapStrings genCmd drvs);

in { inherit rustDep buildRustLibDir; }
