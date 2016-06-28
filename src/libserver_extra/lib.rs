#![crate_name = "server_extra"]

extern crate server_types;

use std::collections::HashMap;
use std::collections::hash_map;
use std::slice;

use server_types::*;


macro_rules! with_value_variants {
    ($mac:ident!($($args:tt)*)) => {
        $mac! {
            $($args)*,
            {
                Bool(bool),
                Int(i64),
                Float(f64),
                Str(String),

                ClientId(ClientId),
                EntityId(EntityId),
                InventoryId(InventoryId),
                PlaneId(PlaneId),
                TerrainChunkId(TerrainChunkId),
                StructureId(StructureId),

                StableClientId(Stable<ClientId>),
                StableEntityId(Stable<EntityId>),
                StableInventoryId(Stable<InventoryId>),
                StablePlaneId(Stable<PlaneId>),
                StableTerrainChunkId(Stable<TerrainChunkId>),
                StableStructureId(Stable<StructureId>),

                V2(V2),
                V3(V3),
                Region2(Region<V2>),
                Region3(Region<V3>),
            }
        }
    };
}

macro_rules! mk_repr {
    ($Repr:ident, { $($vname:ident($vty:ty),)* }) => {
        #[derive(Clone, Debug)]
        enum $Repr {
            Null,
            Array(Vec<Repr>),
            Hash(HashMap<String, Repr>),
            $($vname($vty),)*
        }
    };
}
with_value_variants!(mk_repr!(Repr));

macro_rules! mk_value {
    ($Value:ident, $Repr:ident, { $($vname:ident($vty:ty),)* }) => {
        #[derive(Clone, Debug)]
        pub enum $Value {
            Null,
            $($vname($vty),)*
        }

        impl $Value {
            fn from_repr(repr: $Repr) -> Option<$Value> {
                match repr {
                    $Repr::Null => Some($Value::Null),
                    $Repr::Array(_) => None,
                    $Repr::Hash(_) => None,
                    $(
                        $Repr::$vname(val) => Some($Value::$vname(val)),
                    )*
                }
            }

            fn to_repr(self) -> $Repr {
                match self {
                    $Value::Null => $Repr::Null,
                    $(
                        $Value::$vname(val) => $Repr::$vname(val),
                    )*
                }
            }
        }
    };
}
with_value_variants!(mk_value!(Value, Repr));


pub enum View<'a> {
    Value(Value),
    Array(ArrayView<'a>),
    Hash(HashView<'a>),
}

impl<'a> View<'a> {
    fn from_repr(repr: &'a Repr) -> View<'a> {
        match *repr {
            Repr::Array(ref v) => View::Array(ArrayView::new(v)),
            Repr::Hash(ref h) => View::Hash(HashView::new(h)),
            ref val => View::Value(Value::from_repr(val.clone()).unwrap()),
        }
    }

    pub fn unwrap_value(self) -> Value {
        match self {
            View::Value(x) => x,
            _ => panic!("not a Value"),
        }
    }

    pub fn unwrap_array(self) -> ArrayView<'a> {
        match self {
            View::Array(x) => x,
            _ => panic!("not an Array"),
        }
    }

    pub fn unwrap_hash(self) -> HashView<'a> {
        match self {
            View::Hash(x) => x,
            _ => panic!("not a Hash"),
        }
    }
}

pub enum ViewMut<'a> {
    Value(Value),
    Array(ArrayViewMut<'a>),
    Hash(HashViewMut<'a>),
}

impl<'a> ViewMut<'a> {
    fn from_repr(repr: &'a mut Repr) -> ViewMut<'a> {
        match *repr {
            Repr::Array(ref mut v) => ViewMut::Array(ArrayViewMut::new(v)),
            Repr::Hash(ref mut h) => ViewMut::Hash(HashViewMut::new(h)),
            ref val => ViewMut::Value(Value::from_repr(val.clone()).unwrap()),
        }
    }

    pub fn unwrap_value(self) -> Value {
        match self {
            ViewMut::Value(x) => x,
            _ => panic!("not a Value"),
        }
    }

    pub fn unwrap_array(self) -> ArrayViewMut<'a> {
        match self {
            ViewMut::Array(x) => x,
            _ => panic!("not an Array"),
        }
    }

    pub fn unwrap_hash(self) -> HashViewMut<'a> {
        match self {
            ViewMut::Hash(x) => x,
            _ => panic!("not a Hash"),
        }
    }
}


#[derive(Clone, Copy, Debug)]
pub struct ArrayView<'a> {
    v: &'a Vec<Repr>,
}

impl<'a> ArrayView<'a> {
    fn new(v: &'a Vec<Repr>) -> ArrayView<'a> {
        ArrayView { v: v }
    }

    pub fn get(self, idx: usize) -> View<'a> {
        View::from_repr(&self.v[idx])
    }

    pub fn len(self) -> usize {
        self.v.len()
    }

    pub fn iter(self) -> ArrayViewIter<'a> {
        ArrayViewIter { inner: self.v.iter() }
    }
}

#[derive(Debug)]
pub struct ArrayViewMut<'a> {
    v: &'a mut Vec<Repr>,
}

impl<'a> ArrayViewMut<'a> {
    fn new(v: &'a mut Vec<Repr>) -> ArrayViewMut<'a> {
        ArrayViewMut { v: v }
    }

    pub fn borrow<'b>(&'b mut self) -> ArrayViewMut<'b> {
        ArrayViewMut { v: self.v }
    }

    pub fn get(self, idx: usize) -> View<'a> {
        View::from_repr(&self.v[idx])
    }

    pub fn get_mut(self, idx: usize) -> ViewMut<'a> {
        ViewMut::from_repr(&mut self.v[idx])
    }

    pub fn len(self) -> usize {
        self.v.len()
    }

    pub fn set(self, idx: usize, value: Value) {
        self.v[idx] = value.to_repr();
    }

    pub fn set_array(self, idx: usize) -> ArrayViewMut<'a> {
        let top = self.v;
        top[idx] = Repr::Array(Vec::new());
        if let Repr::Array(ref mut v) = top[idx] {
            ArrayViewMut::new(v)
        } else {
            unreachable!("result of inserting Array should be Array");
        }
    }

    pub fn set_hash(self, idx: usize) -> HashViewMut<'a> {
        let top = self.v;
        top[idx] = Repr::Hash(HashMap::new());
        if let Repr::Hash(ref mut h) = top[idx] {
            HashViewMut::new(h)
        } else {
            unreachable!("result of inserting Hash should be Hash");
        }
    }

    pub fn get_or_set_array(self, idx: usize) -> ArrayViewMut<'a> {
        let is_array = if let Repr::Array(_) = self.v[idx] { true } else { false };
        if is_array {
            self.get_mut(idx).unwrap_array()
        } else {
            self.set_array(idx)
        }

        /* TODO: the obvious implementation seems to trigger a borrow checker bug (?)
        if let Some(&mut Repr::Array(ref mut v)) = top.get_mut(idx) {
            return ArrayViewMut::new(v);
        }
        // Otherwise...
        ArrayViewMut::new(top).set_array(idx)
        */
    }

    pub fn get_or_set_hash(self, idx: usize) -> HashViewMut<'a> {
        let is_hash = if let Repr::Hash(_) = self.v[idx] { true } else { false };
        if is_hash {
            self.get_mut(idx).unwrap_hash()
        } else {
            self.set_hash(idx)
        }
    }

    pub fn push(self) {
        self.v.push(Repr::Null);
    }

    pub fn pop(self) {
        self.v.pop();
    }

    pub fn iter(self) -> ArrayViewIter<'a> {
        ArrayViewIter { inner: self.v.iter() }
    }

    pub fn iter_mut(self) -> ArrayViewIterMut<'a> {
        ArrayViewIterMut { inner: self.v.iter_mut() }
    }
}

pub struct ArrayViewIter<'a> {
    inner: slice::Iter<'a, Repr>,
}

impl<'a> Iterator for ArrayViewIter<'a> {
    type Item = View<'a>;

    fn next(&mut self) -> Option<View<'a>> {
        match self.inner.next() {
            Some(x) => Some(View::from_repr(x)),
            None => None,
        }
    }
}

pub struct ArrayViewIterMut<'a> {
    inner: slice::IterMut<'a, Repr>,
}

impl<'a> Iterator for ArrayViewIterMut<'a> {
    type Item = ViewMut<'a>;

    fn next(&mut self) -> Option<ViewMut<'a>> {
        match self.inner.next() {
            Some(x) => Some(ViewMut::from_repr(x)),
            None => None,
        }
    }
}


#[derive(Clone, Copy, Debug)]
pub struct HashView<'a> {
    h: &'a HashMap<String, Repr>,
}

impl<'a> HashView<'a> {
    fn new(h: &'a HashMap<String, Repr>) -> HashView<'a> {
        HashView { h: h }
    }

    pub fn get(self, key: &str) -> Option<View<'a>> {
        self.h.get(key).map(View::from_repr)
    }

    pub fn contains(self, key: &str) -> bool {
        self.h.contains_key(key)
    }

    pub fn len(self) -> usize {
        self.h.len()
    }

    pub fn iter(self) -> HashViewIter<'a> {
        HashViewIter { inner: self.h.iter() }
    }
}

#[derive(Debug)]
pub struct HashViewMut<'a> {
    h: &'a mut HashMap<String, Repr>,
}

impl<'a> HashViewMut<'a> {
    fn new(h: &'a mut HashMap<String, Repr>) -> HashViewMut<'a> {
        HashViewMut { h: h }
    }

    pub fn borrow<'b>(&'b mut self) -> HashViewMut<'b> {
        HashViewMut { h: self.h }
    }

    pub fn get(self, key: &str) -> Option<View<'a>> {
        self.h.get(key).map(View::from_repr)
    }

    pub fn get_mut(self, key: &str) -> Option<ViewMut<'a>> {
        self.h.get_mut(key).map(ViewMut::from_repr)
    }

    pub fn contains(self, key: &str) -> bool {
        self.h.contains_key(key)
    }

    pub fn len(self) -> usize {
        self.h.len()
    }

    pub fn set(self, key: &str, value: Value) {
        self.h.insert(key.to_owned(), value.to_repr());
    }

    pub fn set_array(self, key: &str) -> ArrayViewMut<'a> {
        self.h.insert(key.to_owned(), Repr::Array(Vec::new()));
        if let Repr::Array(ref mut v) = *self.h.get_mut(key).unwrap() {
            ArrayViewMut::new(v)
        } else {
            unreachable!("result of inserting Array should be Array");
        }
    }

    pub fn set_hash(self, key: &str) -> HashViewMut<'a> {
        self.h.insert(key.to_owned(), Repr::Hash(HashMap::new()));
        if let Repr::Hash(ref mut h) = *self.h.get_mut(key).unwrap() {
            HashViewMut::new(h)
        } else {
            unreachable!("result of inserting Hash should be Hash");
        }
    }

    pub fn get_or_set_array(self, key: &str) -> ArrayViewMut<'a> {
        let is_array = if let Some(&Repr::Array(_)) = self.h.get(key) { true } else { false };
        if is_array {
            self.get_mut(key).unwrap().unwrap_array()
        } else {
            self.set_array(key)
        }
    }

    pub fn get_or_set_hash(self, key: &str) -> HashViewMut<'a> {
        let is_hash = if let Some(&Repr::Hash(_)) = self.h.get(key) { true } else { false };
        if is_hash {
            self.get_mut(key).unwrap().unwrap_hash()
        } else {
            self.set_hash(key)
        }
    }

    pub fn remove(self, key: &str) {
        self.h.remove(key);
    }

    pub fn iter(self) -> HashViewIter<'a> {
        HashViewIter { inner: self.h.iter() }
    }

    pub fn iter_mut(self) -> HashViewIterMut<'a> {
        HashViewIterMut { inner: self.h.iter_mut() }
    }
}

pub struct HashViewIter<'a> {
    inner: hash_map::Iter<'a, String, Repr>,
}

impl<'a> Iterator for HashViewIter<'a> {
    type Item = (&'a str, View<'a>);

    fn next(&mut self) -> Option<(&'a str, View<'a>)> {
        match self.inner.next() {
            Some((k, v)) => Some((k, View::from_repr(v))),
            None => None,
        }
    }
}

pub struct HashViewIterMut<'a> {
    inner: hash_map::IterMut<'a, String, Repr>,
}

impl<'a> Iterator for HashViewIterMut<'a> {
    type Item = (&'a str, ViewMut<'a>);

    fn next(&mut self) -> Option<(&'a str, ViewMut<'a>)> {
        match self.inner.next() {
            Some((k, v)) => Some((k, ViewMut::from_repr(v))),
            None => None,
        }
    }
}


#[derive(Clone)]
pub struct Extra {
    h: Option<Box<HashMap<String, Repr>>>,
}

impl Extra {
    pub fn new() -> Extra {
        Extra {
            h: None,
        }
    }

    pub fn get(&self, key: &str) -> Option<View> {
        if let Some(ref h) = self.h {
            HashView::new(h).get(key)
        } else {
            None
        }
    }

    pub fn get_mut(&mut self, key: &str) -> Option<ViewMut> {
        if let Some(ref mut h) = self.h {
            HashViewMut::new(h).get_mut(key)
        } else {
            None
        }
    }

    pub fn contains(&self, key: &str) -> bool {
        if let Some(ref h) = self.h {
            HashView::new(h).contains(key)
        } else {
            false
        }
    }

    pub fn len(&self) -> usize {
        if let Some(ref h) = self.h {
            h.len()
        } else {
            0
        }
    }

    fn view_mut(&mut self) -> HashViewMut {
        if self.h.is_none() {
            self.h = Some(Box::new(HashMap::new()));
        }
        if let Some(ref mut h) = self.h {
            HashViewMut::new(h)
        } else {
            unreachable!("value after setting to Some should be Some");
        }
    }

    pub fn set(&mut self, key: &str, value: Value) {
        self.view_mut().set(key, value)
    }

    pub fn set_array(&mut self, key: &str) -> ArrayViewMut {
        self.view_mut().set_array(key)
    }

    pub fn set_hash(&mut self, key: &str) -> HashViewMut {
        self.view_mut().set_hash(key)
    }

    pub fn get_or_set_array(&mut self, key: &str) -> ArrayViewMut {
        self.view_mut().get_or_set_array(key)
    }

    pub fn get_or_set_hash(&mut self, key: &str) -> HashViewMut {
        self.view_mut().get_or_set_hash(key)
    }

    pub fn remove(&mut self, key: &str) {
        let clear = 
            if let Some(ref mut h) = self.h {
                HashViewMut::new(h).remove(key);
                h.len() == 0
            } else {
                // It's already empty.
                return;
            };
        if clear {
            self.h = None;
        }
    }

    pub fn iter(&self) -> ExtraIter {
        if let Some(ref h) = self.h {
            ExtraIter { inner: Some(HashView::new(h).iter()) }
        } else {
            ExtraIter { inner: None }
        }
    }

    pub fn iter_mut(&mut self) -> ExtraIterMut {
        if let Some(ref mut h) = self.h {
            ExtraIterMut { inner: Some(HashViewMut::new(h).iter_mut()) }
        } else {
            ExtraIterMut { inner: None }
        }
    }
}

pub struct ExtraIter<'a> {
    inner: Option<HashViewIter<'a>>,
}

impl<'a> Iterator for ExtraIter<'a> {
    type Item = (&'a str, View<'a>);

    fn next(&mut self) -> Option<(&'a str, View<'a>)> {
        if let Some(ref mut inner) = self.inner {
            inner.next()
        } else {
            None
        }
    }
}

pub struct ExtraIterMut<'a> {
    inner: Option<HashViewIterMut<'a>>,
}

impl<'a> Iterator for ExtraIterMut<'a> {
    type Item = (&'a str, ViewMut<'a>);

    fn next(&mut self) -> Option<(&'a str, ViewMut<'a>)> {
        if let Some(ref mut inner) = self.inner {
            inner.next()
        } else {
            None
        }
    }
}
