use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet};
use std::collections::{btree_map, btree_set};
use std::collections::Bound::Included;

use types::*;


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
    // This implementation represents multisets using BTreeSet<(A, B)> instead of
    // HashMap<A, HashSet<B>>.  Turns out inserts are 30-50% faster this way...
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

        for &(_, ref subscriber) in multi_lookup(&self.chan_sub, &channel) {
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

        for &(_, ref subscriber) in multi_lookup(&self.chan_sub, &channel) {
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

        for &(_, ref publisher) in multi_lookup(&self.chan_pub, &channel) {
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

        for &(_, ref publisher) in multi_lookup(&self.chan_pub, &channel) {
            assoc_remove(&mut self.pub_sub,
                         (publisher.clone(), subscriber.clone()),
                         || f(&publisher, &channel, &subscriber));
        }
    }


    pub fn message<F>(&self, publisher: &P, mut f: F)
            where F: FnMut(&P, &S) {
        for (&(_, ref subscriber), _) in multi_lookup_count(&self.pub_sub, publisher) {
            f(publisher, subscriber);
        }
    }


    pub fn subscribe_publisher<F>(&mut self, subscriber: S, publisher: P, mut f: F)
            where F: FnMut(&P, &S) {
        assoc_insert(&mut self.pub_sub,
                     (publisher.clone(), subscriber.clone()),
                     || f(&publisher, &subscriber));
    }

    pub fn unsubscribe_publisher<F>(&mut self, subscriber: S, publisher: P, mut f: F)
            where F: FnMut(&P, &S) {
        assoc_remove(&mut self.pub_sub,
                     (publisher.clone(), subscriber.clone()),
                     || f(&publisher, &subscriber));
    }

    pub fn channel_message<F>(&self, channel: &C, mut f: F)
            where F: FnMut(&C, &S) {
        for &(_, ref subscriber) in multi_lookup(&self.chan_sub, channel) {
            f(channel, subscriber);
        }
    }
}

fn multi_lookup<'a, K: Name, V: Name>(set: &'a BTreeSet<(K, V)>,
                                      k: &K) -> btree_set::Range<'a, (K, V)> {
    set.range((Included(&(k.clone(), V::min_bound())),
               Included(&(k.clone(), V::max_bound()))))
}

fn multi_lookup_count<'a, K1, K2, V>(map: &'a BTreeMap<(K1, K2), V>,
                                     k1: &K1) -> btree_map::Range<'a, (K1, K2), V>
        where K1: Name, K2: Name {
    map.range((Included(&(k1.clone(), K2::min_bound())),
               Included(&(k1.clone(), K2::max_bound()))))
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
                f();
            }
        },
    }
}
