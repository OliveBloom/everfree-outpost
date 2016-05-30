Here is a list of the layout of the src/ directory and what each component does.

## Server

 * `src/libserver_bundle`: Code for reading and writing save bundles.  This is
   separate from the main server to allow writing external save file
   manipulation utilities.  Note that the code for importing/exporting a bundle
   from the World is elsewhere, in `src/server`.
 * `src/libserver_config`: Defines the `Storage` and `Data` types, which track
   various file paths and game data definitions respectively.
 * `src/libserver_extra`: Defines the `Extra` type, which is roughly a
   dynamically-typed data structure for storing additional data from scripts.
 * `src/libserver_types`: Common type and constant definitions used throughout
   the server code.  Most `libserver*`, `libterrain_gen*`, and `server`
   components glob-import this entire library.
 * `src/libserver_util`: Miscellaneous utility definitions for the server.
   Most of these could likely be moved to a common (client/server) library.
 * `src/libserver_world_types`: Defines miscellaneous types associated with
   World objects, such as Attachment enums and Flags.  These are separate from
   `src/server/world` because `libserver_bundle` refers to them.
 * `src/libsyntax_exts`: Defines Rust syntax extensions (procedural macros)
   used in the server.
 * `src/libterrain_gen`: The library responsible for actually generating
   terrain.  The `server` runs this code in a background thread when players
   explore new areas of the map.
 * `src/libterrain_gen_algo`: Support algorithms for terrain generation, such
   as Perlin noise, Poisson disk sampling, and cellular automata simulation.
 * `src/server`: The main server executable.  Most game logic lives here.
 * `src/wrapper`: The server wrapper.  Handles incoming connections and routes
   messages between clients and the backend (`src/server`).

## Client

 * `src/client`: The JS/HTML game client.  Contains `client.html` and all the
   supporting CSS and Javascript.  The Javascript code here is slowly being
   replaced with Rust code in `libclient`.
 * `src/libclient`: The Rust component of the game client.  This gets compiled
   to asm.js (see the `asmlibs` section below) and included with the other
   client Javascript.

## `asmlibs`

 * `src/asmlibs`: Defines the client's asm.js entry points.  Also includes some
   support files for constructing `asmlibs.js`.
 * `src/libasmmalloc`: A simple memory allocator for use with asm.js.
 * `src/libasmrt`: Low-level Rust runtime code for asm.js.  Includes essential
   lang items and macros.
 * `src/libfakestd`: A drop-in replacement for `libstd`, but containing only
   the definitions that are re-exports from `libcore` or `libcollections`, so
   it's usable with asm.js.  Some libraries use this to achieve portability
   between asm.js and full Rust environments (though they must restrict
   themselves to the asm.js-compatible parts of the `libstd`/`libfakestd` API).

## Common

 * `src/libphysics`: The core physics engine.  Performs collision detection and
   nothing else.  This library also contains the definitions of the `V2`, `V3`,
   and `Region` types, which are used pervasively throughout the server and
   client.

## Miscellaneous

 * `src/gen`: Scripts for generating various files.  In particular,
   `src/gen/data` contains the infrastructure used to generate the game data
   definitions from `data/*.py`.
 * `src/migrations`: Tools for migrating save files from one server version to
   another.
 * `src/test_terrain_gen`: A Python API to `libterrain_gen`, to support
   connecting terrain generation it directly to the map viewer.  This allows
   for testing terrain generation without rebuilding the entire server each
   time.
 * `src/uvedit`: Sprite mask editor.  The masks drawn with this tool are used
   to generate equipment sprites (currently only socks).
