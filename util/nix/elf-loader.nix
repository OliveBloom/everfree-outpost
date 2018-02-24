{ stdenv, lib, runCommand, patchelf, ... }:

let pathDrv = runCommand "get-elf-loader"
        { patchelf = "${patchelf}/bin/patchelf"; }
        "${stdenv.shell} ${./get-elf-loader.sh}";
    path' = builtins.readFile pathDrv;
    path = lib.removeSuffix "\n" path';
in
    # None of the checked files were present and dynamically linked.  Don't
    # adjust the interpreter.
    if path == "" then null
    else if lib.hasPrefix "/lib" path then path
    # Found an interpreter, but it didn't start with /lib.  Probably means
    # we're on NixOS, and shouldn't adjust the interpreter.
    else null
