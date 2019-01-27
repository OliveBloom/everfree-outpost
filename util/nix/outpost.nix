{ rev }:

with builtins;

let deps = import ./deps.nix;
    inherit (deps.all) rustDeps rustSrc bitflagsSrc;


in deps.pkgs.stdenv.mkDerivation rec {
    name = "everfree-outpost-${version}";
    version = "0.0.0";

    src = fetchGit {
        url = ../../.git;
        inherit rev;
    };

    buildInputs = attrValues deps.build;
    propagatedBuildInputs = attrValues deps.run;

    inherit rustDeps rustSrc bitflagsSrc;

    configurePhase = ''
        ./configure --release \
            --rust-extra-libdir=${rustDeps} \
            --rust-home=${rustSrc} \
            --bitflags-home=${bitflagsSrc}
    '';

    installPhase = ''
        mkdir -p $out
        cp -av dist/* $out
    '';
}
