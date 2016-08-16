use libphysics::CHUNK_SIZE;

use types::*;
use util::SmallSet;

use components;
use engine::split2::Coded;
use logic;
use world::Structure;
use world::bundle::{Importer, Exporter};
use world::bundle::{import, export};
use world::bundle::types as b;
use world::object::*;



pub fn structure_area(s: ObjectRef<Structure>) -> SmallSet<V2> {
    let mut area = SmallSet::new();
    for p in s.bounds().reduce().div_round_signed(CHUNK_SIZE).points() {
        area.insert(p);
    }

    area
}


engine_part2!(pub EngineLifecycle(
        world, cache, vision, messages, dialogs,
        Components));


struct ImportVisitor<'a, 'd: 'a>(&'a mut EngineLifecycle<'d>);

impl<'a, 'd> import::Visitor for ImportVisitor<'a, 'd> {
    fn visit_client(&mut self, _id: ClientId, _b: &b::Client) {
    }

    fn visit_entity(&mut self, id: EntityId, b: &b::Entity) {
        logic::entity::on_create(self.0.refine(), id);
        components::import_entity(self.0.refine(), id, b);
    }

    fn visit_inventory(&mut self, _id: InventoryId, _b: &b::Inventory) {
    }

    fn visit_plane(&mut self, _id: PlaneId, _b: &b::Plane) {
    }

    fn visit_terrain_chunk(&mut self, id: TerrainChunkId, _b: &b::TerrainChunk) {
        logic::terrain_chunk::on_create(self.0.refine(), id);
    }

    fn visit_structure(&mut self, id: StructureId, _b: &b::Structure) {
        logic::structure::on_create(self.0.refine(), id);
        logic::structure::on_import(self.0.refine(), id);
    }
}

/// Hook to be called after importing some new game objects.
pub fn on_import(eng: &mut EngineLifecycle, importer: &Importer, bundle: &b::Bundle) {
    importer.visit_imports(bundle, &mut ImportVisitor(eng));
}


// TODO: separate "export" logic from "destroy" logic
struct ExportVisitor<'a, 'd: 'a>(&'a mut EngineLifecycle<'d>);

impl<'a, 'd> export::Visitor for ExportVisitor<'a, 'd> {
    fn visit_client(&mut self, _id: ClientId, _b: &mut b::Client) {
    }

    fn visit_entity(&mut self, id: EntityId, b: &mut b::Entity) {
        components::export_entity(self.0.refine(), id, b);

        logic::entity::on_destroy(self.0.refine(), id);
        components::cleanup_entity(self.0.refine(), id);
    }

    fn visit_inventory(&mut self, _id: InventoryId, _b: &mut b::Inventory) {
    }

    fn visit_plane(&mut self, _id: PlaneId, _b: &mut b::Plane) {
    }

    fn visit_terrain_chunk(&mut self, id: TerrainChunkId, _b: &mut b::TerrainChunk) {
        logic::terrain_chunk::on_destroy(self.0.refine(), id);
    }

    fn visit_structure(&mut self, id: StructureId, _b: &mut b::Structure) {
        logic::structure::on_destroy(self.0.refine(), id);
    }
}

/// Hook to be called before deleting some exported game objects..
pub fn on_export(eng: &mut EngineLifecycle, exporter: &mut Exporter) {
    exporter.visit_exports(&mut ExportVisitor(eng));
}
