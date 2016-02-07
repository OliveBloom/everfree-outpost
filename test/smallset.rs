extern crate server_util;

use server_util::SmallSet;

fn insert_remove_n(n: u32) {
    let mut v: SmallSet<u32> = SmallSet::new();
    println!(" == insert_remove {} ==", n);
    for i in 0 .. n {
        println!("insert {}", i);
        v.insert(i);
    }
    for i in 0 .. n {
        println!("insert {}", i);
        v.insert(i);
    }
    for i in 0 .. n {
        assert!(v.contains(&i));
    }
    println!(" ---");
    for i in 0 .. n {
        println!("remove {}", i);
        v.remove(&i);
    }
}

fn main() {
    for n in 1 .. 11 {
        insert_remove_n(n);
    }
}
