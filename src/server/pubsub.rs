use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};
use std::collections::hash_map;
use std::collections::{BTreeMap, BTreeSet};
use std::collections::btree_map;
use std::collections::Bound::Included;
use std::hash::Hash;

use types::*;

use util::{multimap_insert, multimap_remove};
use util::OptionIterExt;
use util::RefcountedMap;



pub trait Name: Ord+Clone {
    fn min_bound() -> Self;
    fn max_bound() -> Self;
}

macro_rules! prim_name {
    ($($x:ident,)*) => {
        $(
            impl Name for $x {
                fn min_bound() -> $x { ::std::$x::MIN }
                fn max_bound() -> $x { ::std::$x::MAX }
            }
        )*
    };
}
prim_name!(
    u8, u16, u32, u64, usize,
    i8, i16, i32, i64, isize,
);

macro_rules! id_name {
    ($($x:ident,)*) => {
        $(
            impl Name for $x {
                fn min_bound() -> $x { $x(Name::min_bound()) }
                fn max_bound() -> $x { $x(Name::max_bound()) }
            }
        )*
    };
}
id_name!(
    WireId,
    ClientId,
    EntityId,
    InventoryId,
    PlaneId,
    TerrainChunkId,
    StructureId,
);

macro_rules! tuple_name {
    ($($A:ident,)*) => {
        impl<$($A: Name,)*> Name for ($($A,)*) {
            fn min_bound() -> ($($A,)*) { ($(<$A as Name>::min_bound(),)*) }
            fn max_bound() -> ($($A,)*) { ($(<$A as Name>::max_bound(),)*) }
        }
    };
}
tuple_name!();
tuple_name!(A,);
tuple_name!(A, B,);
tuple_name!(A, B, C,);
tuple_name!(A, B, C, D,);
tuple_name!(A, B, C, D, E,);

impl Name for V2 {
    fn min_bound() -> V2 { scalar(Name::min_bound()) }
    fn max_bound() -> V2 { scalar(Name::max_bound()) }
}

impl Name for V3 {
    fn min_bound() -> V3 { scalar(Name::min_bound()) }
    fn max_bound() -> V3 { scalar(Name::max_bound()) }
}


#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct ZOrdered<V: Vn>(pub V);

impl<V: Vn+Eq> Ord for ZOrdered<V> {
    fn cmp(&self, other: &ZOrdered<V>) -> Ordering {
        let mut ord = Ordering::Equal;
        let mut min_lz = 32;

        V::fold_axes((), |a, ()| {
            let lz = (self.0.get(a) ^ other.0.get(a)).leading_zeros();
            if lz < min_lz {
                min_lz = lz;
                ord = self.0.get(a).cmp(&other.0.get(a));
            }
        });

        ord
    }
}

impl<V: Vn+Eq> PartialOrd for ZOrdered<V> {
    fn partial_cmp(&self, other: &ZOrdered<V>) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<V: Vn+Eq+Clone> Name for ZOrdered<V> {
    fn min_bound() -> ZOrdered<V> {
        ZOrdered(scalar(Name::min_bound()))
    }

    fn max_bound() -> ZOrdered<V> {
        ZOrdered(scalar(Name::max_bound()))
    }
}



/// A slightly unusual publish/subscribe system.
///
/// Each subscriber can associate itself with any number of channels.  Each publisher can associate
/// itself with channels in the same way.  When a publisher sends a message, each subscriber that
/// is associated with at least one of its channels will receive a single copy of the message.
/// Furthermore, when a publisher/subscriber becomes un/associated with a channel, it is notified
/// of the identities of all the subscribers/publishers currently associated with that channel.
pub struct PubSub<P: Name, C: Name, S: Name> {
    chan_pub: BTreeSet<(C, P)>,
    chan_sub: BTreeSet<(C, S)>,
    pub_sub: BTreeMap<(P, S), usize>,
}

impl<P: Name, C: Name, S: Name> PubSub<P, C, S> {
    pub fn new() -> PubSub<P, C, S> {
        PubSub {
            chan_pub: BTreeSet::new(),
            chan_sub: BTreeSet::new(),
            pub_sub: BTreeMap::new(),
        }
    }

    pub fn publish<F>(&mut self, publisher: P, channel: C, mut f: F)
            where F: FnMut(&P, &C, &S) {
        if !self.chan_pub.insert((channel.clone(), publisher.clone())) {
            return;
        }

        for &(_, ref subscriber) in self.chan_sub.range(Included(&(channel.clone(), S::min_bound())),
                                                        Included(&(channel.clone(), S::max_bound()))) {
            assoc_insert(&mut self.pub_sub,
                         (publisher.clone(), subscriber.clone()),
                         || f(&publisher, &channel, &subscriber));
        }
    }

    pub fn unpublish<F>(&mut self, publisher: P, channel: C, mut f: F)
            where F: FnMut(&P, &C, &S) {
        if !self.chan_pub.remove(&(channel.clone(), publisher.clone())) {
            return;
        }

        for &(_, ref subscriber) in self.chan_sub.range(Included(&(channel.clone(), S::min_bound())),
                                                        Included(&(channel.clone(), S::max_bound()))) {
            assoc_remove(&mut self.pub_sub,
                         (publisher.clone(), subscriber.clone()),
                         || f(&publisher, &channel, &subscriber));
        }
    }

    pub fn subscribe<F>(&mut self, subscriber: S, channel: C, mut f: F)
            where F: FnMut(&P, &C, &S) {
        if !self.chan_sub.insert((channel.clone(), subscriber.clone())) {
            return;
        }

        for &(_, ref publisher) in self.chan_pub.range(Included(&(channel.clone(), P::min_bound())),
                                                       Included(&(channel.clone(), P::max_bound()))) {
            assoc_insert(&mut self.pub_sub,
                         (publisher.clone(), subscriber.clone()),
                         || f(&publisher, &channel, &subscriber));
        }
    }

    pub fn unsubscribe<F>(&mut self, subscriber: S, channel: C, mut f: F)
            where F: FnMut(&P, &C, &S) {
        if !self.chan_sub.remove(&(channel.clone(), subscriber.clone())) {
            return;
        }

        for &(_, ref publisher) in self.chan_pub.range(Included(&(channel.clone(), P::min_bound())),
                                                       Included(&(channel.clone(), P::max_bound()))) {
            assoc_remove(&mut self.pub_sub,
                         (publisher.clone(), subscriber.clone()),
                         || f(&publisher, &channel, &subscriber));
        }
    }


    pub fn message<F>(&self, publisher: &P, mut f: F)
            where F: FnMut(&P, &S) {
        for (&(_, ref subscriber), _) in self.pub_sub.range(Included(&(publisher.clone(), S::min_bound())),
                                                            Included(&(publisher.clone(), S::max_bound()))) {
            f(publisher, subscriber);
        }
    }
}

fn assoc_insert<K, F>(map: &mut BTreeMap<K, usize>, k: K, f: F)
        where K: Ord, F: FnOnce() {
    *map.entry(k).or_insert_with(|| { f(); 0 }) += 1;
}

fn assoc_remove<K, F>(map: &mut BTreeMap<K, usize>, k: K, f: F)
        where K: Ord, F: FnOnce() {
    match map.entry(k) {
        btree_map::Entry::Vacant(_) => panic!("assoc_remove: key not found"),
        btree_map::Entry::Occupied(mut e) => {
            *e.get_mut() -= 1;
            if *e.get() == 0 {
                e.remove();
            }
        },
    }
}




/*
pub trait Name: Eq+Hash+Clone {}

/// A slightly unusual publish/subscribe system.
///
/// Each subscriber can associate itself with any number of channels.  Each publisher can associate
/// itself with channels in the same way.  When a publisher sends a message, each subscriber that
/// is associated with at least one of its channels will receive a single copy of the message.
/// Furthermore, when a publisher/subscriber becomes un/associated with a channel, it is notified
/// of the identities of all the subscribers/publishers currently associated with that channel.
pub struct PubSub<P: Name, C: Name, S: Name> {
    chan_pub: HashMap<C, HashSet<P>>,
    chan_sub: HashMap<C, HashSet<S>>,
    pub_sub: HashMap<P, RefcountedMap<S, ()>>,
}

impl<P: Name, C: Name, S: Name> PubSub<P, C, S> {
    pub fn new() -> PubSub<P, C, S> {
        PubSub {
            chan_pub: HashMap::new(),
            chan_sub: HashMap::new(),
            pub_sub: HashMap::new(),
        }
    }

    pub fn publish<F>(&mut self, publisher: P, channel: C, mut f: F)
            where F: FnMut(&P, &C, &S) {
        if self.chan_pub.get(&channel).map(|pubs| pubs.contains(&publisher)) == Some(true) {
            return;
        }

        for subscriber in self.chan_sub.get(&channel).map(|subs| subs.iter()).unwrap_iter() {
            assoc_insert(&mut self.pub_sub,
                         publisher.clone(),
                         subscriber.clone(),
                         || f(&publisher, &channel, &subscriber));
        }

        multimap_insert(&mut self.chan_pub, channel, publisher);
    }

    pub fn unpublish<F>(&mut self, publisher: P, channel: C, mut f: F)
            where F: FnMut(&P, &C, &S) {
        if self.chan_pub.get(&channel).map(|pubs| pubs.contains(&publisher)) != Some(true) {
            return;
        }

        for subscriber in self.chan_sub.get(&channel).map(|subs| subs.iter()).unwrap_iter() {
            assoc_remove(&mut self.pub_sub,
                         publisher.clone(),
                         subscriber.clone(),
                         || f(&publisher, &channel, &subscriber));
        }

        multimap_remove(&mut self.chan_pub, channel, publisher);
    }

    pub fn subscribe<F>(&mut self, subscriber: S, channel: C, mut f: F)
            where F: FnMut(&P, &C, &S) {
        if self.chan_sub.get(&channel).map(|subs| subs.contains(&subscriber)) == Some(true) {
            return;
        }

        for publisher in self.chan_pub.get(&channel).map(|pubs| pubs.iter()).unwrap_iter() {
            assoc_insert(&mut self.pub_sub,
                         publisher.clone(),
                         subscriber.clone(),
                         || f(&publisher, &channel, &subscriber));
        }

        multimap_insert(&mut self.chan_sub, channel, subscriber);
    }

    pub fn unsubscribe<F>(&mut self, subscriber: S, channel: C, mut f: F)
            where F: FnMut(&P, &C, &S) {
        if self.chan_sub.get(&channel).map(|subs| subs.contains(&subscriber)) != Some(true) {
            return;
        }

        for publisher in self.chan_pub.get(&channel).map(|pubs| pubs.iter()).unwrap_iter() {
            assoc_remove(&mut self.pub_sub,
                         publisher.clone(),
                         subscriber.clone(),
                         || f(&publisher, &channel, &subscriber));
        }

        multimap_remove(&mut self.chan_sub, channel, subscriber);
    }


    pub fn message<F>(&self, publisher: &P, mut f: F)
            where F: FnMut(&P, &S) {
        for (subscriber, _) in self.pub_sub.get(publisher).map(|subs| subs.iter()).unwrap_iter() {
            f(publisher, subscriber);
        }
    }
}


fn assoc_insert<K, V, F>(map: &mut HashMap<K, RefcountedMap<V, ()>>, k: K, v: V, f: F)
        where K: Eq+Hash,
              V: Eq+Hash,
              F: FnOnce() {
    map.entry(k).or_insert_with(|| RefcountedMap::new()).retain(v, f);
}

fn assoc_remove<K, V, F>(map: &mut HashMap<K, RefcountedMap<V, ()>>, k: K, v: V, f: F)
        where K: Eq+Hash,
              V: Eq+Hash,
              F: FnOnce() {
    match map.entry(k) {
        hash_map::Entry::Vacant(_) => panic!("assoc_remove: key not found"),
        hash_map::Entry::Occupied(mut e) => {
            e.get_mut().release(v, |()| f());
            if e.get().len() == 0 {
                e.remove();
            }
        },
    }
}

*/
