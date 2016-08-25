use rustc_serialize::json::Json;

pub use self::block::{BlockData, BlockDef};
pub use self::item::ItemData;
pub use self::recipe::{Recipe, RecipeData};
pub use self::template::{StructureTemplate, StructureTemplates};
pub use self::animation::{Animation, AnimationData};
pub use self::sprite_layer::SpriteLayerData;
pub use self::loot_table::LootTables;


#[derive(Debug)]
pub struct ParseError(pub String);


pub struct Data {
    pub block_data: BlockData,
    pub item_data: ItemData,
    pub recipes: RecipeData,
    pub structure_templates: StructureTemplates,
    pub animations: AnimationData,
    pub sprite_layers: SpriteLayerData,
    pub loot_tables: LootTables,
}

impl Data {
    pub fn from_json(block_json: Json,
                     item_json: Json,
                     recipe_json: Json,
                     structure_template_json: Json,
                     animation_json: Json,
                     sprite_layer_json: Json,
                     loot_table_json: Json) -> Result<Data, ParseError> {
        Ok(Data {
            block_data: try!(BlockData::from_json(block_json)),
            item_data: try!(ItemData::from_json(item_json)),
            recipes: try!(RecipeData::from_json(recipe_json)),
            structure_templates: try!(StructureTemplates::from_json(structure_template_json)),
            animations: try!(AnimationData::from_json(animation_json)),
            sprite_layers: try!(SpriteLayerData::from_json(sprite_layer_json)),
            loot_tables: try!(LootTables::from_json(loot_table_json)),
        })
    }
}


macro_rules! fail {
    ($msg:expr) => {
        fail!($msg,)
    };
    ($msg:expr, $($extra:tt)*) => {
        Err(ParseError(format!($msg, $($extra)*)))
    };
}

macro_rules! expect {
    ($e:expr, $str:expr) => {
        expect!($e, $str,)
    };
    ($e:expr, $str:expr, $($extra:tt)*) => {
        match $e {
            Some(x) => x,
            None => return Err(ParseError(format!($str, $($extra)*))),
        }
    };
}

macro_rules! find_convert {
    ($json:expr, $key:expr, $convert:ident, $where_:expr) => {
        find_convert!($json, $key, $convert, $where_,)
    };
    ($json:expr, $key:expr, $convert:ident, $where_:expr, $($extra:tt)*) => {{
        let key = $key;
        match $json.find(key) {
            Some(j) => match j.$convert() {
                Some(x) => Ok(x),
                None => fail!(concat!("failed to convert key \"{}\" with {} ", $where_),
                              key, stringify!($convert), $($extra)*),
            },
            None => fail!(concat!("missing key \"{}\" ", $where_),
                          key, $($extra)*),
        }
    }};
}

macro_rules! get_convert {
    ($json:expr, $key:expr, $convert:ident, $where_:expr) => {
        get_convert!($json, $key, $convert, $where_,)
    };
    ($json:expr, $key:expr, $convert:ident, $where_:expr, $($extra:tt)*) => {
        try!(find_convert!($json, $key, $convert, $where_, $($extra)*))
    };
}

macro_rules! convert {
    ($json:expr, $convert:ident, $what:expr) => {
        convert!($expr, $convert, $what,)
    };
    ($json:expr, $convert:ident, $what:expr, $($extra:tt)*) => {{
        match json.$convert() {
            Some(x) => Ok(x),
            None => fail!(concat!("failed to convert ", $what, " with {}"),
                          $($extra)*, stringify!($convert)),
        },
    }};
}


pub mod block;
pub mod item;
pub mod recipe;
pub mod template;
pub mod animation;
pub mod sprite_layer;
pub mod loot_table;
