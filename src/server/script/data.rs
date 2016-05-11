use types::*;

use data::Data;
use python::{PyBox, PyResult};

use super::Pack;
use super::rust_ref::{RustRef, RustRefType};

macro_rules! data_ref_func {
    ( $($all:tt)* ) => ( rust_ref_func!(Data, $($all)*); );
}

define_python_class! {
    class DataRef: RustRef {
        type_obj DATA_REF_TYPE;
        initializer init;
        accessor get_type;
        method_macro data_ref_func!;


        fn block_count(&this) -> usize {
            this.block_data.len()
        }

        fn block_by_name(&this, name: String) -> PyResult<BlockId> {
            Ok(pyunwrap!(this.block_data.find_id(&name),
                         key_error, "no such block: {:?}", name))
        }

        fn get_block_by_name(&this, name: String) -> Option<BlockId> {
            this.block_data.find_id(&name)
        }

        fn block_name(&this, id: BlockId) -> Option<PyResult<PyBox>> {
            this.block_data.get_name(id).map(|s| Pack::pack(s))
        }

        fn block_shape(&this, id: BlockId) -> Option<&'static str> {
            if (id as usize) < this.block_data.len() {
                let desc = match this.block_data.shape(id) {
                    Shape::Empty => "empty",
                    Shape::Floor => "floor",
                    Shape::Solid => "solid",
                    //Shape::RampE => "ramp_e",
                    //Shape::RampW => "ramp_w",
                    //Shape::RampS => "ramp_s",
                    Shape::RampN => "ramp_n",
                };
                Some(desc)
            } else {
                None
            }
        }


        fn item_count(&this) -> usize {
            this.item_data.len()
        }

        fn item_by_name(&this, name: String) -> PyResult<ItemId> {
            Ok(pyunwrap!(this.item_data.find_id(&name),
                         key_error, "no such item: {:?}", name))
        }

        fn get_item_by_name(&this, name: String) -> Option<ItemId> {
            this.item_data.find_id(&name)
        }

        fn item_name(&this, id: ItemId) -> Option<PyResult<PyBox>> {
            this.item_data.get_name(id).map(|s| Pack::pack(s))
        }


        fn recipe_count(&this) -> usize {
            this.recipes.len()
        }

        fn recipe_by_name(&this, name: String) -> PyResult<RecipeId> {
            Ok(pyunwrap!(this.recipes.find_id(&name),
                         key_error, "no such recipe: {:?}", name))
        }

        fn get_recipe_by_name(&this, name: String) -> Option<RecipeId> {
            this.recipes.find_id(&name)
        }

        fn recipe_name(&this, id: RecipeId) -> Option<PyResult<PyBox>> {
            this.recipes.get_recipe(id).map(|r| Pack::pack(&r.name as &str))
        }

        fn recipe_inputs(&this, id: RecipeId) -> Option<PyResult<PyBox>> {
            this.recipes.get_recipe(id).map(|r| Pack::pack(r.inputs.clone()))
        }

        fn recipe_outputs(&this, id: RecipeId) -> Option<PyResult<PyBox>> {
            this.recipes.get_recipe(id).map(|r| Pack::pack(r.outputs.clone()))
        }

        fn recipe_station(&this, id: RecipeId) -> Option<Option<TemplateId>> {
            this.recipes.get_recipe(id).map(|r| r.station)
        }


        // TODO: clean up Option<PyResult<_>> methods above to work more like template_name &
        // template_layer below


        fn template_count(&this) -> usize {
            this.structure_templates.len()
        }

        fn template_by_name(&this, name: String) -> PyResult<TemplateId> {
            Ok(pyunwrap!(this.structure_templates.find_id(&name),
                         key_error, "no such template: {:?}", name))
        }

        fn get_template_by_name(&this, name: String) -> Option<TemplateId> {
            this.structure_templates.find_id(&name)
        }

        fn template_name(&this, id: TemplateId) -> PyResult<PyBox> {
            let t = pyunwrap!(this.structure_templates.get_template(id),
                              runtime_error, "no template with that ID");
            Pack::pack(&t.name as &str)
        }

        fn template_layer(&this, id: TemplateId) -> PyResult<u8> {
            let t = pyunwrap!(this.structure_templates.get_template(id),
                              runtime_error, "no template with that ID");
            Ok(t.layer)
        }


        fn animation_count(&this) -> usize {
            this.animations.len()
        }

        fn animation_by_name(&this, name: String) -> PyResult<AnimId> {
            Ok(pyunwrap!(this.animations.find_id(&name),
                         key_error, "no such animation: {:?}", name))
        }

        fn get_animation_by_name(&this, name: String) -> Option<AnimId> {
            this.animations.find_id(&name)
        }

        fn animation_name(&this, id: AnimId) -> PyResult<PyBox> {
            let a = pyunwrap!(this.animations.get_animation(id),
                              runtime_error, "no animation with that ID");
            Pack::pack(&a.name as &str)
        }

        fn animation_framerate(&this, id: AnimId) -> PyResult<u32> {
            let a = pyunwrap!(this.animations.get_animation(id),
                              runtime_error, "no animation with that ID");
            Ok(a.framerate)
        }

        fn animation_length(&this, id: AnimId) -> PyResult<u32> {
            let a = pyunwrap!(this.animations.get_animation(id),
                              runtime_error, "no animation with that ID");
            Ok(a.length)
        }


        fn sprite_layer_count(&this) -> usize {
            this.sprite_layers.len()
        }

        fn sprite_layer_by_name(&this, name: String) -> PyResult<u32> {
            Ok(pyunwrap!(this.sprite_layers.find_id(&name),
                         key_error, "no such sprite layer: {:?}", name))
        }

        fn get_sprite_layer_by_name(&this, name: String) -> Option<u32> {
            this.sprite_layers.find_id(&name)
        }

        fn sprite_layer_name(&this, id: u32) -> PyResult<PyBox> {
            let l = pyunwrap!(this.sprite_layers.get_name(id),
                              runtime_error, "no sprite layer with that ID");
            Pack::pack(l)
        }
    }
}

unsafe impl RustRefType for Data {
    fn get_type_object() -> PyBox {
        get_type().to_box()
    }
}
