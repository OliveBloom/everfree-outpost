use server_config::Data;
use server_config::data::RecipeItem;
use server_types::*;

use api::{PyBox, PyResult};
use conv::Pack;
use rust_ref::{RustRef, RustRefType};


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
            this.blocks().len()
        }

        fn block_by_name(&this, name: String) -> PyResult<BlockId> {
            Ok(pyunwrap!(this.get_block_id(&name),
                         key_error, "no such block: {:?}", name))
        }

        fn get_block_by_name(&this, name: String) -> Option<BlockId> {
            this.get_block_id(&name)
        }

        fn block_name(&this, id: BlockId) -> PyResult<PyBox> {
            let b = pyunwrap!(this.get_block(id),
                              runtime_error, "no block with that ID");
            Pack::pack(b.name())
        }

        fn block_shape(&this, id: BlockId) -> PyResult<&'static str> {
            let b = pyunwrap!(this.get_block(id),
                              runtime_error, "no block with that ID");
            let desc = match b.flags().shape() {
                Shape::Empty => "empty",
                Shape::Floor => "floor",
                Shape::Solid => "solid",
                //Shape::RampE => "ramp_e",
                //Shape::RampW => "ramp_w",
                //Shape::RampS => "ramp_s",
                Shape::RampN => "ramp_n",
            };
            Ok(desc)
        }


        fn item_count(&this) -> usize {
            this.items().len()
        }

        fn item_by_name(&this, name: String) -> PyResult<ItemId> {
            Ok(pyunwrap!(this.get_item_id(&name),
                         key_error, "no such item: {:?}", name))
        }

        fn get_item_by_name(&this, name: String) -> Option<ItemId> {
            this.get_item_id(&name)
        }

        fn item_name(&this, id: ItemId) -> PyResult<PyBox> {
            let i = pyunwrap!(this.get_item(id),
                              runtime_error, "no item with that ID");
            Pack::pack(i.name())
        }


        fn recipe_count(&this) -> usize {
            this.recipes().len()
        }

        fn recipe_by_name(&this, name: String) -> PyResult<RecipeId> {
            Ok(pyunwrap!(this.get_recipe_id(&name),
                         key_error, "no such recipe: {:?}", name))
        }

        fn get_recipe_by_name(&this, name: String) -> Option<RecipeId> {
            this.get_recipe_id(&name)
        }

        fn recipe_name(&this, id: RecipeId) -> PyResult<PyBox> {
            let r = pyunwrap!(this.get_recipe(id),
                              runtime_error, "no recipe with that ID");
            Pack::pack(r.name())
        }

        fn recipe_inputs(&this, id: RecipeId) -> PyResult<PyBox> {
            let r = pyunwrap!(this.get_recipe(id),
                              runtime_error, "no recipe with that ID");
            Pack::pack(r.inputs())
        }

        fn recipe_outputs(&this, id: RecipeId) -> PyResult<PyBox> {
            let r = pyunwrap!(this.get_recipe(id),
                              runtime_error, "no recipe with that ID");
            Pack::pack(r.outputs())
        }

        fn recipe_station(&this, id: RecipeId) -> PyResult<TemplateId> {
            let r = pyunwrap!(this.get_recipe(id),
                              runtime_error, "no recipe with that ID");
            Ok(r.station)
        }


        // TODO: clean up Option<PyResult<_>> methods above to work more like template_name &
        // template_layer below


        fn template_count(&this) -> usize {
            this.templates().len()
        }

        fn template_by_name(&this, name: String) -> PyResult<TemplateId> {
            Ok(pyunwrap!(this.get_template_id(&name),
                         key_error, "no such template: {:?}", name))
        }

        fn get_template_by_name(&this, name: String) -> Option<TemplateId> {
            this.get_template_id(&name)
        }

        fn template_name(&this, id: TemplateId) -> PyResult<PyBox> {
            let t = pyunwrap!(this.get_template(id),
                              runtime_error, "no template with that ID");
            Pack::pack(t.name())
        }

        fn template_layer(&this, id: TemplateId) -> PyResult<u8> {
            let t = pyunwrap!(this.get_template(id),
                              runtime_error, "no template with that ID");
            Ok(t.layer)
        }

        fn template_size(&this, id: TemplateId) -> PyResult<V3> {
            let t = pyunwrap!(this.get_template(id),
                              runtime_error, "no template with that ID");
            Ok(t.size)
        }


        fn animation_count(&this) -> usize {
            this.animations().len()
        }

        fn animation_by_name(&this, name: String) -> PyResult<AnimId> {
            Ok(pyunwrap!(this.get_animation_id(&name),
                         key_error, "no such animation: {:?}", name))
        }

        fn get_animation_by_name(&this, name: String) -> Option<AnimId> {
            this.get_animation_id(&name)
        }

        fn animation_name(&this, id: AnimId) -> PyResult<PyBox> {
            let a = pyunwrap!(this.get_animation(id),
                              runtime_error, "no animation with that ID");
            Pack::pack(a.name())
        }

        fn animation_framerate(&this, id: AnimId) -> PyResult<u32> {
            let a = pyunwrap!(this.get_animation(id),
                              runtime_error, "no animation with that ID");
            Ok(a.framerate as u32)
        }

        fn animation_length(&this, id: AnimId) -> PyResult<u32> {
            let a = pyunwrap!(this.get_animation(id),
                              runtime_error, "no animation with that ID");
            Ok(a.length as u32)
        }
    }
}

impl Pack for RecipeItem {
    fn pack(self) -> PyResult<PyBox> {
        Pack::pack((self.item, self.quantity))
    }
}

unsafe impl RustRefType for Data {
    fn get_type_object() -> PyBox {
        get_type().to_box()
    }
}
