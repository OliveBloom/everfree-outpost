local action = require('core.action')
local util = require('core.util')
local door = require('outpost.lib.door')

local COLORS = { 'red', 'orange', 'yellow', 'green', 'blue', 'purple' }
local COLOR_VAL = {
    red = 0,
    orange = 1,
    yellow = 2,
    green = 3,
    blue = 4,
    purple = 5,
    empty = -1,
}

local function get_puzzle(plane, pid)
    local puzzles = plane:extra().puzzles
    if puzzles == nil then
        puzzles = {}
        plane:extra().puzzles = puzzles
    end

    local p = puzzles[pid]
    if p == nil then
        p = {
            door = nil,
            door_open = false,
            slots = {},
        }
        puzzles[pid] = p
    end

    return p
end

local function apply_gem_puzzle_slot(s, k, v)
    local pid, slot, init = v:match('(.*),(.*),(.*)')
    local p = get_puzzle(s:plane(), pid)
    print('slot', s, k, v, p)
    slot = 1 + slot
    p.slots[slot] = COLOR_VAL[init]
    s:extra().puzzle_id = pid
    s:extra().slot_index = slot
end

local function apply_gem_puzzle_door(s, k, v)
    local pid = v
    local p = get_puzzle(s:plane(), pid)
    print('door', s, k, v, p)
    p.door = s:stable_id()
    s:extra().puzzle_id = pid
end


return {
    apply_gem_puzzle_slot = apply_gem_puzzle_slot,
    apply_gem_puzzle_door = apply_gem_puzzle_door,
}
