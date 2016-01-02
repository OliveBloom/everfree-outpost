def hit_pos(e):
    return e.pos() + 16 + e.facing() * 32

def hit_tile(e):
    return hit_pos(e).px_to_tile()

def hit_structure(e):
    return e.plane().find_structure_at_point(hit_tile(e))
