from outpost_data.core.image2 import loader, Image, Anim
# The `pony_sprite` module contains lots of pony-specific helper functions.
from outpost_data.outpost.lib import pony_sprite
# Other miscellaneous image manipulation functions for sprites.
from outpost_data.outpost.lib import sprite_util

def init():
    # Build a function to load sprites from assets/sprites/.
    sprites = loader('sprites', unit=pony_sprite.SPRITE_SIZE)

    # Get the pony SpriteDef object.
    pony = pony_sprite.get_pony_sprite()

    # Add the sit and sleep animation definitions.  These give the length and
    # framerate of each animation. but have no associated image data.

    # *Note about Directions*: Most animations, such as "walk" and "sit", come
    # in 8 varieties, one for each of the 8 directions.  The directions used by
    # the game engine start with 0 = east and run clockwise:
    #
    #   5 6 7
    #    \|/
    #   4-*-0
    #    /|\
    #   3 2 1
    #
    # However, to reduce the amount of artwork needed, each of the west-facing
    # directions (3, 4, 5) is rendered as a mirrored version of its east-facing
    # counterpart (1, 0, 7).  So there are only five directions present in the
    # actual sprite sheets.  These (unfortunately) currently use a completely
    # different numbering scheme, based on the structure of the original
    # MLPonline sprite sheets:
    #
    #   x 0 1
    #    \|/
    #   x-*-2
    #    /|\
    #   x 4 3
    #
    # The result is that sprite definitions need a little extra code to map
    # back and forth between game-directions and spritesheet-directions.  In
    # particular, the `DIRS` table maps game directions to spritesheet
    # directions (using `sd = DIRS[gd].idx`), and the `INV_DIRS` table does the
    # reverse (`gd = INV_DIRS[sd]`).


    # Use a library function to define the 8 "sit" animations.  This sets the
    # length and framerate for each animation, but does not include any images.
    # In this case the length is 1 frame and the rate is 1 FPS.
    pony_sprite.make_anim_dirs(pony, 'sit', 1, 1)


    # Define the "sleep" animation.  This doesn't have different directions,
    # but it does have 6 frames.
    sleep_anim = pony.add_anim('sleep', 6, 2)

    # Since the sleep animation doesn't have -0..-7 directional variants, we
    # need to specifically register its direction.  Otherwise, the wrong
    # hat/eye graphics will be used during this animation.
    pony_sprite.register_anim_facing(sleep_anim, 0)


    # Now we need to provide images for the new animations.
    #
    # Image data works like this: Imagine a big table, where the columns are
    # labeled with animations ("walk", "sit", "sleep", etc.) and the rows are
    # labeled with components of the sprite ("unicorn body", "tail #1", "witch
    # hat", etc.).  Every cell of the table must contain some image data,
    # otherwise there will be nothing to show for that component when that
    # particular animation is playing.  We have just added some new columns,
    # and we have to fill in all the image data for those columns.

    # We add the "sit" animation data first.

    # Iterate over spritesheet directions
    for sd in range(5):
        # Get the corresponding game direction
        gd = pony_sprite.INV_DIRS[sd]

        # The name of the current animation
        anim_name = 'sit-%d' % gd

        # Handle both male and female variants
        for sex in ('m', 'f'):
            ms = 'mare' if sex == 'f' else 'stallion'

            # "Base" (pony body) part.  The images for this part are actually
            # made up of separate body, horn, and wing layers.  A library
            # function combines the layers as needed to make E/P/U/A variants.
            part = pony.get_part('%s/base' % sex)

            layer_images = {
                    layer: sprites('base/%s/%s-%d-%s.png' % (ms, ms, sd, layer))
                            .extract((1, 0))
                    for layer in pony_sprite.LAYER_NAMES
                    }
            tribe_images = pony_sprite.make_tribe_sheets(layer_images)

            for tribe, img in tribe_images.items():
                anim = img.sheet_to_anim((1, 1), 1)
                # The "base" part has a variant for each tribe ("E", "P", etc)
                part.get_variant(tribe).add_graphics(anim_name, anim)

            # Mane and tail parts.  These are simpler than the base/body,
            # though there are several variations to iterate over.
            for mane_or_tail, index in pony_sprite.standard_manes_tails():
                part = pony.get_part('%s/%s' % (sex, mane_or_tail))
                img = sprites('parts/%s/%s%d.png' % (ms, mane_or_tail, index)) \
                        .extract((5 + sd, 0))
                # Always set depth for mane/tail to 120.  This controls the
                # layering order for different parts of the sprite.
                img = sprite_util.set_depth(img, 120)
                anim = img.sheet_to_anim((1, 1), 1)
                # The mane and tail parts have variants "1", "2", "3".
                part.get_variant('%d' % index).add_graphics(anim_name, anim)

            # Hat box part.  The hat box indicates the position of the head
            # within the sprite, so that hats, eyes, etc. can all be placed
            # automatically at the right location.
            part = pony.get_part('%s/_dummy' % sex)
            img = sprites('base/%s/%s-%d-hat-box.png' % (ms, ms, sd)) \
                    .extract((1, 0))
            anim = img.sheet_to_anim((1, 1), 1)
            variant = part.get_variant('hat_box').add_graphics(anim_name, anim)

    # Now add the "sleep" animation data.

    # Handle both male and female variants
    for sex in ('m', 'f'):
        ms = 'mare' if sex == 'f' else 'stallion'

        sheet = sprites('sleep-%s.png' % sex)

        # The "sleep" sprite sheets have five rows of three frames each.
        # First, cut apart the rows.
        rows = [sheet.extract((0, y), size=(3, 1)) for y in range(5)]

        # Use the first four rows to build the tribe base sheets.
        tribe_images = pony_sprite.make_tribe_sheets({
            'base': rows[0],
            'horn': rows[1],
            'frontwing': rows[2],
            'backwing': rows[3],
            })

        # Define a helper function to turn a sheet into an anim.  We can't use
        # the normal `sheet_to_anim()` because we need to loop the frames in
        # order 0-0-1-2-2-1.
        def make_anim(sheet):
            frames = [sheet.extract((x, 0)) for x in (0, 0, 1, 2, 2, 1)]
            return Anim(frames, rate=2)

        part = pony.get_part('%s/base' % sex)
        # Now we can add the base anims.
        for tribe, img in tribe_images.items():
            anim = make_anim(img)
            part.get_variant(tribe).add_graphics('sleep', anim)

        # Add the hat box, from the bottom row of the "sleep" sheet.
        part = pony.get_part('%s/_dummy' % sex)
        part.get_variant('hat_box').add_graphics('sleep', make_anim(rows[4]))


        # Mane and tail parts.  These are in the original MLPonline layout, so
        # the sleep animation frames are located at (10, 0).
        for mane_or_tail, index in pony_sprite.standard_manes_tails():
            part = pony.get_part('%s/%s' % (sex, mane_or_tail))
            img = sprites('parts/%s/%s%d.png' % (ms, mane_or_tail, index)) \
                    .extract((10, 0), size=(3, 1))
            img = sprite_util.set_depth(img, 120)
            anim = make_anim(img)
            part.get_variant('%d' % index).add_graphics('sleep', anim)

        # Eye part.  We add a blank animation so that the closed eyes from the
        # underlying sprite sheet will be visible.
        blank_frame = Image((1, 1), unit=pony_sprite.SPRITE_SIZE)
        blank_anim = Anim([blank_frame] * 6, rate=2)
        part = pony.get_part('%s/eyes' % sex)
        # TODO: kind of a hack - this trick of looping over the variants only
        # works if the variants are defined before this code runs.
        for v in part.iter_variants():
            v.add_graphics('sleep', blank_anim)
