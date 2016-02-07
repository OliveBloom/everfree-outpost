extern crate server_util;

use server_util::SmallVec;

fn push_pop_n(n: u32) {
    let mut v: SmallVec<u32> = SmallVec::new();
    println!(" == push/pop {} ==", n);
    for i in 0 .. n {
        println!("push {}", i);
        v.push(i);
    }
    println!(" ---");
    for i in 0 .. n {
        println!("pop {}", i);
        v.pop();
    }
}

fn main() {
    for n in 1 .. 11 {
        push_pop_n(n);
    }
}
