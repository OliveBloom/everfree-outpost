from configure.checks.context import ConfigError

def configure(ctx):
    out = ctx.file('exe')

    ctx.detect('pandoc', 'Pandoc', ('pandoc',), chk_pandoc)

    ctx.copy_arg('nix_patch_elf_loader', 'ELF loader override')
    if ctx.args.nix_patch_elf_loader is not None:
        ctx.detect('patchelf', 'PatchELF', ('patchelf',), chk_patchelf)

def requirements(ctx):
    reqs = ('pandoc',)
    if ctx.args.nix_patch_elf_loader is not None:
        reqs += ('patchelf',)
    return reqs


def chk_pandoc(ctx, pandoc):
    if not ctx.run(pandoc, ('--version',)):
        raise ConfigError('not found')
    return True

def chk_patchelf(ctx, patchelf):
    if not ctx.run(patchelf, ('--version',)):
        raise ConfigError('not found')
    return True
