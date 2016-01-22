# Dependencies

First, you need to install the game's dependencies.  Most of these are packages
you should get from your Linux distro repositories.  This list uses Debian
names, so if you're not on a Debian derivative (Ubuntu, Mint, etc) you should
look up the corresponding package for your distro.

The Outpost build system is somewhat modular, in that it can reuse components
from a pre-built Outpost package so that you can avoid installing the
dependencies needed to build those components.  The list of dependencies below
is categorized based on what part of the project you want to work on.

To run the game at all, get these packages:
 * `libpython3.4`
 * `libsqlite3-0`
 * `socat`

If you want to work on mods (new item/structure/ability definitions and
corresponding server-side scripts), get these:
 * `ninja`
 * `python3`
 * `python3-pillow`
 * `python3-yaml`

If you want to work on client-side (Javascript) code, get these:
 * `closure-compiler`

If you want to work on server-side (Rust) code, get these:
 * `build-essential`
 * `libpython3.4-dev`
 * `libsqlite3-dev`
 * `libwebsocketpp-dev`
 * `libboost-dev`
 * `libboost-system-dev`
 * [outpost-build-tools](http://play.everfree-outpost.com/outpost-build-tools-2015-12-07.tar.xz)
   OR build from source the Rust compiler and packages listed in README.md.

If you want to work on the physics or graphics libraries, get these:
 * `yui-compressor`
 * Build from source the versions of `emscripten-fastcomp` and
   `rust-emscripten-passes` listed in README.md.
 * Source checkouts of `rust-lang/rust` and `rust-lang-nursery/bitflags`, at
   the versions listed in README.md.

If you skipped any section above:
 * Download the most recent Linux build and unpack it alongside your source
   checkout.  The URL will be based on the name of the most recent `release-*`
   tag in the git repository, and follows the format of:
   http://play.everfree-outpost.com/outpost-linux-2016-01-16a.tar.xz


# Configuration

The basic configuration command looks like this:

    ./configure --debug --mods=cornucopia,ore_vein

If you want to work only on mods, add these additional flags:

        --prebuilt-dir=../outpost-linux-2016-01-16a --data-only --force

If you want to build a specific part other than mods, see `./configure --help`
for other flags, particularly `--use-prebuilt`.

Some errors will appear during configuration if any dependency is missing (for
example, if you skipped installing something because you won't work on the part
of the game that requires it), but it should continue anyway due to `--force`.


# Building

Run `ninja`.  If you get an error that's not caused by a change you made, first
make sure you have all the necessary dependencies and rerun `configure`, and if
that doesn't work, post the exact `configure` command you ran and its output.


# Running

Open two different terminals in the `everfree-outpost/dist/` directory. Run
`python3 -m http.server 8889` in one (to get an HTTP server to serve up the
client to a web browser) and run `bin/run_server.sh` in the other (to run the
Everfree Outpost game server). Then point a web browser to
`http://localhost:8889/www/client.html` and you should get the usual game.

To make yourself a superuser on your server, first join the game, then run this
command in another terminal:

    cd /path/to/everfree-outpost/dist
    echo 'eng.client_by_name("Your Name").extra()["superuser"] = True' | socat - unix:repl

Afterward, you can use `/help` in-game to see the new superuser commands.

To shut down or restart the server cleanly, run:

    cd /path/to/everfree-outpost/dist
    echo restart_server | socat - unix:repl     # restart
    echo shutdown | socat - unix:repl           # shutdown

In particular, use the restart command after compiling new changes in order to
see the effect in-game.
