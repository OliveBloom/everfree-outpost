use std::collections::HashSet;
use std::{i32, u32};
use test::{Bencher, black_box};
use rand::{Rng, XorShiftRng, random};

use types::*;
use util::SmallSet;

use pubsub::{PubSub, Name, ZOrdered};
use vision::{Vision, NoHooks, vision_region};


struct BlackBoxHooks;

impl ::vision::Hooks for BlackBoxHooks {
    fn on_structure_template_change(&mut self, cid: ClientId, sid: StructureId) {
        black_box((cid, sid));
    }
}


fn vision_init(rng: &mut XorShiftRng) -> Vision {
    let mut v = Vision::new();

    for i in 0 .. 10000 {
        let pos = V2::new(rng.gen_range(0, 100),
                          rng.gen_range(0, 100));
        let dir = V2::new(rng.gen_range(-1, 2),
                          rng.gen_range(-1, 2));

        let mut area = SmallSet::new();
        area.insert(pos);
        area.insert(pos + dir);
        v.add_structure(StructureId(i), PlaneId(2), area, &mut NoHooks);
    }

    for i in 0 .. 1000 {
        let pos = V3::new(rng.gen_range(0, 100 * 512),
                          rng.gen_range(0, 100 * 512),
                          0);
        v.add_client(ClientId(i), PlaneId(2), vision_region(pos), &mut NoHooks);
    }

    v
}

#[bench]
fn client_add_remove(b: &mut Bencher) {
    let mut rng: XorShiftRng = random();
    let mut v = vision_init(&mut rng);

    b.iter(|| {
        let id = rng.gen_range(1000, 2000);
        let pos = V3::new(rng.gen_range(0, 100 * 512),
                          rng.gen_range(0, 100 * 512),
                          0);
        v.add_client(ClientId(id), PlaneId(2), vision_region(pos), &mut NoHooks);
        v.remove_client(ClientId(id), &mut NoHooks);
    });
}

#[bench]
fn client_add(b: &mut Bencher) {
    let mut rng: XorShiftRng = random();
    let mut v = vision_init(&mut rng);

    b.iter(|| {
        for id in 1000 .. 2000 {
            let pos = V3::new(rng.gen_range(0, 100 * 512),
                              rng.gen_range(0, 100 * 512),
                              0);
            v.add_client(ClientId(id), PlaneId(2), vision_region(pos), &mut NoHooks);
            v.remove_client(ClientId(id), &mut NoHooks);
        }
    });
}

#[bench]
fn client_message(b: &mut Bencher) {
    let mut rng: XorShiftRng = random();
    let mut v = vision_init(&mut rng);

    b.iter(|| {
        let id = rng.gen_range(0, 10000);
        v.change_structure_template(StructureId(id), &mut BlackBoxHooks);
    });
}


fn pubsub_init(rng: &mut XorShiftRng) -> PubSub<u32, (PlaneId, V2), u32> {
    let mut ps = PubSub::new();

    for i in 0 .. 10000 {
        let pos = V2::new(rng.gen_range(0, 100),
                          rng.gen_range(0, 100));
        let dir = V2::new(rng.gen_range(-1, 2),
                          rng.gen_range(-1, 2));

        ps.publish(i, (PlaneId(2), pos), |_,_,_| ());
        ps.publish(i, (PlaneId(2), pos + dir), |_,_,_| ());
    }

    for i in 0 .. 1000 {
        let pos = V3::new(rng.gen_range(0, 100 * 512),
                          rng.gen_range(0, 100 * 512),
                          0);
        for p in vision_region(pos).points() {
            ps.subscribe(i, (PlaneId(2), p), |_,_,_| ());
        }
    }

    ps
}

#[bench]
fn pubsub_add_remove(b: &mut Bencher) {
    let mut rng: XorShiftRng = random();
    let mut ps = pubsub_init(&mut rng);

    b.iter(|| {
        let id = rng.gen_range(1000, 2000);
        let pos = V3::new(rng.gen_range(0, 100 * 512),
                          rng.gen_range(0, 100 * 512),
                          0);
        for p in vision_region(pos).points() {
            ps.subscribe(id, (PlaneId(2), p), |_,_,_| ());
        }
        for p in vision_region(pos).points() {
            ps.unsubscribe(id, (PlaneId(2), p), |_,_,_| ());
        }
    });
}

#[bench]
fn pubsub_add(b: &mut Bencher) {
    let mut rng: XorShiftRng = random();
    let mut ps = pubsub_init(&mut rng);

    b.iter(|| {
        for id in 1000 .. 2000 {
            let pos = V3::new(rng.gen_range(0, 100 * 512),
                              rng.gen_range(0, 100 * 512),
                              0);
            for p in vision_region(pos).points() {
                ps.subscribe(id, (PlaneId(2), p), |_,_,_| ());
            }
        }
    });
}

#[bench]
fn pubsub_message(b: &mut Bencher) {
    let mut rng: XorShiftRng = random();
    let mut ps = pubsub_init(&mut rng);

    b.iter(|| {
        let id = rng.gen_range(0, 10000);
        ps.message(&id, |&p,&s| { black_box((p, s)); });
    });
}


#[test]
fn pubsub_no_dupes() {
    let mut ps = PubSub::new();

    // There are two channels, `0` and `1`.  Subscriber `S` subscribes to `C` if `S & (1 << C)`,
    // and similarly for publishers.
    ps.subscribe(1, 0, |_,_,_| ());
    ps.subscribe(2, 1, |_,_,_| ());
    ps.subscribe(3, 0, |_,_,_| ());
    ps.subscribe(3, 1, |_,_,_| ());

    ps.publish(1, 0, |_,_,_| ());
    ps.publish(2, 1, |_,_,_| ());
    ps.publish(3, 0, |_,_,_| ());
    ps.publish(3, 1, |_,_,_| ());

    let mut messages = HashSet::new();
    for i in 0 .. 4 {
        ps.message(&i, |&p, &s| {
            assert!(!messages.contains(&(p, s)));
            messages.insert((p, s));
        });
    }

    let expected = vec![
        (1, 1), (1, 3),
        (2, 2), (2, 3),
        (3, 1), (3, 2), (3, 3),
    ].into_iter().collect::<HashSet<_>>();
    assert_eq!(messages, expected);
}


#[test]
fn pubsub_unsubscribe() {
    let mut ps = PubSub::new();

    ps.publish(0, 0, |_,_,_| ());
    ps.publish(1, 1, |_,_,_| ());

    let mut called = false;
    ps.subscribe(0, 0, |&p,&c,&s| { called = true; assert_eq!((p,c,s), (0,0,0)); });
    assert!(called);

    let mut called = false;
    ps.subscribe(0, 1, |&p,&c,&s| { called = true; assert_eq!((p,c,s), (1,1,0)); });
    assert!(called);

    let mut seen = HashSet::new();
    ps.message(&0, |&p, &s| { seen.insert((p, s)); });
    ps.message(&1, |&p, &s| { seen.insert((p, s)); });
    let expected = vec![(0, 0), (1, 0)].into_iter().collect::<HashSet<_>>();
    assert_eq!(seen, expected);


    let mut called = false;
    ps.unsubscribe(0, 1, |&p,&c,&s| { called = true; assert_eq!((p,c,s), (1,1,0)); });
    assert!(called);

    let mut seen = HashSet::new();
    ps.message(&0, |&p, &s| { seen.insert((p, s)); });
    ps.message(&1, |&p, &s| { seen.insert((p, s)); });
    let expected = vec![(0, 0)].into_iter().collect::<HashSet<_>>();
    assert_eq!(seen, expected);
}


#[test]
fn pubsub_unpublish() {
    let mut ps = PubSub::new();

    ps.subscribe(0, 0, |_,_,_| ());
    ps.subscribe(1, 1, |_,_,_| ());

    let mut called = false;
    ps.publish(0, 0, |&p,&c,&s| { called = true; assert_eq!((p,c,s), (0,0,0)); });
    assert!(called);

    let mut called = false;
    ps.publish(0, 1, |&p,&c,&s| { called = true; assert_eq!((p,c,s), (0,1,1)); });
    assert!(called);

    let mut seen = HashSet::new();
    ps.message(&0, |&p, &s| { seen.insert((p, s)); });
    let expected = vec![(0, 0), (0, 1)].into_iter().collect::<HashSet<_>>();
    assert_eq!(seen, expected);


    let mut called = false;
    ps.unpublish(0, 1, |&p,&c,&s| { called = true; assert_eq!((p,c,s), (0,1,1)); });
    assert!(called);

    let mut seen = HashSet::new();
    ps.message(&0, |&p, &s| { seen.insert((p, s)); });
    let expected = vec![(0, 0)].into_iter().collect::<HashSet<_>>();
    assert_eq!(seen, expected);
}
