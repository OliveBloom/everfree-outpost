use types::*;

use data::Data;
use python::PyBox;

use super::Pack;
use super::rust_ref::{RustRef, GetTypeObject};

macro_rules! data_ref_func {
    ( $($all:tt)* ) => ( rust_ref_func!(Data, $($all)*) );
}

define_python_class! {
    class DataRef: RustRef {
        type_obj DATA_REF_TYPE;
        initializer init;
        accessor get_type;
        method_macro data_ref_func!;


        fn item_count(&this) -> usize {
            this.item_data.len()
        }

        fn item_by_name(&this, name: String) -> Option<ItemId> {
            this.item_data.find_id(&name)
        }

        fn item_name(&this, id: ItemId) -> Option<PyBox> {
            this.item_data.get_name(id).map(|s| Pack::pack(s))
        }


        fn recipe_count(&this) -> usize {
            this.recipes.len()
        }

        fn recipe_by_name(&this, name: String) -> Option<RecipeId> {
            this.recipes.find_id(&name)
        }

        fn recipe_name(&this, id: RecipeId) -> Option<PyBox> {
            this.recipes.get_recipe(id).map(|r| Pack::pack(&r.name as &str))
        }

        fn recipe_inputs(&this, id: RecipeId) -> Option<PyBox> {
            this.recipes.get_recipe(id).map(|r| Pack::pack(&r.inputs))
        }

        fn recipe_outputs(&this, id: RecipeId) -> Option<PyBox> {
            this.recipes.get_recipe(id).map(|r| Pack::pack(&r.outputs))
        }

        fn recipe_station(&this, id: RecipeId) -> Option<Option<TemplateId>> {
            this.recipes.get_recipe(id).map(|r| r.station)
        }


        fn template_count(&this) -> usize {
            this.structure_templates.len()
        }

        fn template_by_name(&this, name: String) -> Option<TemplateId> {
            this.structure_templates.find_id(&name)
        }

        fn template_name(&this, id: TemplateId) -> Option<PyBox> {
            this.structure_templates.get_template(id).map(|t| Pack::pack(&t.name as &str))
        }
    }
}

unsafe impl GetTypeObject for Data {
    fn get_type_object() -> PyBox {
        get_type().to_box()
    }
}
