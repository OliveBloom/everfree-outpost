
with builtins;

let deps = import ./deps.nix;
    inherit (deps.all) rustDeps rustSrc bitflagsSrc;

in deps.pkgs.mkShell {
    buildInputs = attrValues deps.all;

    inherit rustDeps rustSrc bitflagsSrc;

    # Convenient access to the nixpkgs we're actually using.
    nixpkgs = deps.pkgs.path;

    OUTPOST_CONFIGURE_ARGS =
        "--rust-extra-libdir=${rustDeps} " +
        "--rust-home=${rustSrc} " +
        "--bitflags-home=${bitflagsSrc} " +
        (if deps.elfLoader == null then "" else
            "--nix-patch-elf-loader=" + deps.elfLoader + " ");
}
