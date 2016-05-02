The `/eval` superuser command runs a bit of Python code on the server.  The
code has access to some special variables for convenience:

 * `eng`: A reference to the `EngineProxy` object representing the game engine.
 * `c`: Your `ClientProxy` object.  Invoke `c("name")` to obtain the client
   whose name is "name".
 * `e`: The `EntityProxy` for your character, equivalent to `c.pawn()`.  Most
   gameplay data is attached to the entity, not the client.  Invoke `e('name')`
   to get the pawn of another client, equivalent to `c('name').pawn()`.
 * `s`: The structure (`StructureProxy`) directly in front of your character.
 * `p`: The plane (`PlaneProxy`) your character is currently on.
 * `b`: The block (`data.BlockProxy`) directly in front of your character.

In addition to these variables, some modules are imported implicitly,
equivalent to:

    from outpost_server.core.data import DATA
    from outpost_server.core.engine import *
    from outpost_server.core.types import *
    from outpost_server.core import util

