with builtins;
let
    branch = "release-18.09";
    date = "2019-01-26";
    rev = "09218fe5de4e85d182a1e55386996cd284ad4049";
    sha256 = "1mmrnyp80jfvqdsal9728h8y3pmyx5snvgls0zfmbjl4qm3m8f76";

    src = fetchTarball {
        name = "nixpkgs-${branch}-${date}-${rev}";
        url = "https://github.com/nixos/nixpkgs/archive/${rev}.tar.gz";
        inherit sha256;
    };

in import src
