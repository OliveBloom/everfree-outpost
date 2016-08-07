use std::prelude::v1::*;
use common_types::*;

use extra_arg::ExtraArg;
use types::*;


protocol! {
    protocol Request [request_op::Opcode = u16] {
        [0x0003] Ping { cookie: u16 },
        [0x0004] Input { when: LocalTime, bits: u16 },
        [0x0009] CraftRecipe {
            station: StructureId, inventory: InventoryId, recipe: RecipeId, count: u16 },
        [0x000a] Chat { msg: String },
        [0x000c] Interact { when: LocalTime },
        [0x000d] UseItem { when: LocalTime, item: ItemId },
        [0x000e] UseAbility { when: LocalTime, ability: ItemId },
        [0x0010] InteractWithArgs { when: LocalTime, arg: ExtraArg },
        [0x0011] UseItemWithArgs { when: LocalTime, ability: ItemId, arg: ExtraArg },
        [0x0012] UseAbilityWithArgs { when: LocalTime, ability: ItemId, arg: ExtraArg },
        [0x0013] MoveItem {
            inv1: InventoryId, slot1: SlotId, inv2: InventoryId, slot2: SlotId, count: u8 },
        [0x0015] CreateCharacter { appearance: u32 },
        [0x0016] Ready {__: ()},
        [0x0017] CloseDialog {__: ()},
    }
}

protocol! {
    protocol Response [response_op::Opcode = u16] {
        [0x8001] TerrainChunk { idx: u16, data: Vec<u16> },
        [0x8003] Pong { cookie: u16, now: LocalTime },
        [0x8005] Init {
            pawn_id: EntityId, now: LocalTime, day_night_base: u32, day_night_ms: u32 },
        [0x8006] KickReason { msg: String },
        [0x8007] UnloadChunk { idx: u16 },
        [0x8008] OpenDialog { which: u32, args: Vec<u32> },
        [0x800a] OpenCrafting { kind: TemplateId, station: StructureId, inventory: InventoryId },
        [0x800b] ChatUpdate { msg: String },
        [0x800c] EntityAppear { id: EntityId, appearance: u32, name: String },
        [0x800d] EntityGone { id: EntityId, when: LocalTime },
        [0x800f] StructureAppear { id: StructureId, template: TemplateId, pos: LocalPos },
        [0x8010] StructureGone { id: StructureId },
        [0x8011] MainInventory { id: InventoryId },
        [0x8012] AbilityInventory { id: InventoryId },
        [0x8013] PlaneFlags { flags: u32 },
        [0x8014] GetInteractArgs { dialog: u32, arg: ExtraArg },
        [0x8015] GetUseItemArgs { item: ItemId, dialog: u32, arg: ExtraArg },
        [0x8016] GetUseAbilityArgs { ability: ItemId, dialog: u32, arg: ExtraArg },
        [0x8017] SyncStatus { status: u8 },
        [0x8018] StructureReplace { id: StructureId, template: TemplateId },
        [0x8019] InventoryUpdate { id: InventoryId, slot: u8, item: (u8, u8, ItemId) },
        [0x801a] InventoryAppear { id: InventoryId, items: Vec<(u8, u8, ItemId)> },
        [0x801b] InventoryGone { id: InventoryId },
        [0x801c] EntityMotionStart { id: EntityId,
            pos: LocalPos, start_time: LocalTime, velocity: LocalOffset, anim: AnimId },
        [0x801d] EntityMotionEnd { id: EntityId, end_time: LocalTime },
        [0x801e] EntityMotionStartEnd { id: EntityId,
            pos: LocalPos, start_time: LocalTime, velocity: LocalOffset, anim: AnimId,
            end_time: LocalTime },
        [0x801f] ProcessedInputs { when: LocalTime, count: u16 },
        [0x8020] ActivityChange { activity: u8 },
        [0x8023] InitNoPawn {
            pos: LocalPos, now: LocalTime, day_night_base: u32, day_night_ms: u32 },
        [0x8024] OpenPonyEdit { name: String },
        [0x8025] EntityActivityIcon { id: EntityId, icon: AnimId },
        [0x8026] CancelDialog { __: () },
        [0x8027] EnergyUpdate { cur: u16, max: u16, rate: (i16, u16), time: LocalTime },
    }
}

