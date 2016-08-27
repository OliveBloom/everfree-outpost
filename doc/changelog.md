# Release 2016-08-27

New features:

 * The inventory dialog now shows the name and description of the selected
   item.
 * Lighting is now rendered using a different formula, which produces
   smoother-looking results.

# Release 2016-08-20

This release adds metal tools:

 * Picks and axes now come in Stone, Copper, and Iron varieties.  Higher-tier
   tools are faster at mining rocks and chopping trees.
 * A Copper Pick or better is required to mine walls in caves.

Crafting has also been somewhat reorganized:

 * New structure: Workbench.  Most recipes are crafted here (instead of at the
   Anvil).
 * New structure: Furnace.  Used to craft metal bars from ore.
 * The Anvil is now used only for crafting copper and iron tools.

Other major changes:

 * Movement code has been rewritten.  There should be almost no lag now when
   moving normally.
 * New structure: Celestial Chest.  Provides storage that is specific to your
   character, but accessible from any celestial chest.
 * New structure: Crate.  A container that changes appearance to reflect what
   items it contains.  Currently only vegetables are supported.

Minor changes:

 * Added an energy bar to the UI.  Buffs that use energy will be added later.
 * Converted crafting UI to WebGL.
 * Added support for `scale_ui` config setting to the new UI code.  The UI is
   now rendered at a multiple of the world scale, determined by `scale_ui`.
 * Added a link to configedit.html to the server list


# Release 2016-07-25

 * Start of changelog
