use std::collections::HashMap;
use rand::Rng;
use rustc_serialize::json::Json;

use libserver_types::*;

use loot::{TableIndex, Weight, Chance, ItemTable, StructureTable};


#[derive(Debug)]
pub struct ParseError(pub String);


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


pub struct LootTables {
    pub item: Box<[ItemTable]>,
    pub item_by_name: HashMap<String, TableIndex>,
    pub structure: Box<[StructureTable]>,
    pub structure_by_name: HashMap<String, TableIndex>,
}

impl LootTables {
    pub fn from_json(json: Json) -> Result<LootTables, ParseError> {
        let items = get_convert!(json, "items", as_array,
                                 "in top-level object");
        let mut item_tables = Vec::with_capacity(items.len());
        let mut item_by_name = HashMap::new();
        for (i, table) in items.iter().enumerate() {
            let ty = get_convert!(table, "type", as_string,
                                  "for item table {}", i);

            let t = 
                match ty {
                    "object" => {
                        let item_id = get_convert!(table, "id", as_i64,
                                                   "for item table {}", i);
                        let min_count = get_convert!(table, "min_count", as_i64,
                                                     "for item table {}", i);
                        let max_count = get_convert!(table, "max_count", as_i64,
                                                     "for item table {}", i);
                        ItemTable::Item(item_id as ItemId, min_count as u8, max_count as u8)
                    },
                    "choose" => {
                        let variants_json = get_convert!(table, "variants", as_array,
                                                         "for item table {}", i);
                        let mut variants = Vec::with_capacity(variants_json.len());
                        let mut weight_sum = 0;
                        for (j, v) in variants_json.iter().enumerate() {
                            let id = get_convert!(v, "id", as_i64,
                                                  "for variant {} of item table {}", j, i);
                            let weight = get_convert!(v, "weight", as_i64,
                                                      "for variant {} of item table {}", j, i);
                            variants.push((id as TableIndex, weight as Weight));
                            weight_sum += weight as i32;
                        }
                        ItemTable::Choose(variants, weight_sum)
                    },
                    "multi" => {
                        let parts_json = get_convert!(table, "parts", as_array,
                                                      "for item table {}", i);
                        let mut parts = Vec::with_capacity(parts_json.len());
                        for (j, v) in parts_json.iter().enumerate() {
                            let id = get_convert!(v, "id", as_i64,
                                                  "for part {} of item table {}", j, i);
                            let chance = get_convert!(v, "chance", as_i64,
                                                      "for part {} of item table {}", j, i);
                            parts.push((id as TableIndex, chance as Chance));
                        }
                        ItemTable::Multi(parts)
                    },
                    _ => return fail!("bad type \"{}\" for item table {}", ty, i),
                };
            item_tables.push(t);

            match table.find("name").and_then(|n| n.as_string()) {
                Some(name) => { item_by_name.insert(name.to_owned(), i as TableIndex); },
                None => {},
            }
        }


        let structures = get_convert!(json, "structures", as_array,
                                      "in top-level object");
        let mut structure_tables = Vec::with_capacity(structures.len());
        let mut structure_by_name = HashMap::new();
        for (i, table) in structures.iter().enumerate() {
            let ty = get_convert!(table, "type", as_string,
                                  "for structure table {}", i);

            let t = 
                match ty {
                    "object" => {
                        let structure_id = get_convert!(table, "id", as_i64,
                                                        "for structure table {}", i);
                        StructureTable::Structure(structure_id as TemplateId)
                    },
                    "choose" => {
                        let variants_json = get_convert!(table, "variants", as_array,
                                                         "for structure table {}", i);
                        let mut variants = Vec::with_capacity(variants_json.len());
                        let mut weight_sum = 0;
                        for (j, v) in variants_json.iter().enumerate() {
                            let id = get_convert!(v, "id", as_i64,
                                                  "for variant {} of structure table {}", j, i);
                            let weight = get_convert!(v, "weight", as_i64,
                                                      "for variant {} of structure table {}", j, i);
                            variants.push((id as TableIndex, weight as Weight));
                            weight_sum += weight as i32;
                        }
                        StructureTable::Choose(variants, weight_sum)
                    },
                    _ => return fail!("bad type \"{}\" for structure table {}", ty, i),
                };
            structure_tables.push(t);

            match table.find("name").and_then(|n| n.as_string()) {
                Some(name) => { structure_by_name.insert(name.to_owned(), i as TableIndex); },
                None => {},
            }
        }


        Ok(LootTables {
            item: item_tables.into_boxed_slice(),
            item_by_name: item_by_name,
            structure: structure_tables.into_boxed_slice(),
            structure_by_name: structure_by_name,
        })
    }

    pub fn eval_item_table<R: Rng>(&self, rng: &mut R, name: &str) -> Vec<(ItemId, u8)> {
        let mut result = Vec::new();
        let id = self.item_by_name[name];
        self.item[id as usize].eval(&self.item, rng, &mut result);
        result
    }

    pub fn eval_structure_table<R: Rng>(&self, rng: &mut R, name: &str) -> Option<TemplateId> {
        let mut result = None;
        let id = self.structure_by_name[name];
        self.structure[id as usize].eval(&self.structure, rng, &mut result);
        result
    }
}
