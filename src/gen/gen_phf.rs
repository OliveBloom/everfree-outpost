/// Generate parameters for a perfect hash function.  Takes a list of string keys (one per line) on
/// stdin, and writes the parameters to stdout.
///
/// The output consist of three lines:
///  1. The number of buckets `b`, hash table modulus `m >= b`, and intermediate table size `r`.
///  2. The bucket assignment `hash(s) % m` for each input.
///  3. The PHF parameter values `l_1 ... l_r`.
/// Numbers on each line are separated by spaces.
///
/// The modulus and intermediate table size are always powers of two.
use std::cmp;
use std::hash::{Hash, Hasher};
#[allow(deprecated)] use std::hash::SipHasher;
use std::io::{self, BufReader, BufRead};
use std::iter;
extern crate rand;

#[allow(deprecated)]    // for SipHasher
fn hash<H: Hash+?Sized>(s: &H, k: (u64, u64)) -> u64 {
    let mut h = SipHasher::new_with_keys(k.0, k.1);
    s.hash(&mut h);
    h.finish()
}

fn phf(s: &str, ls: &[u64], r: u64, m: u64) -> u64 {
    let h1 = hash(s, (0x123456, 0xfedcba));
    let l = ls[(h1 % r) as usize];
    let h2 = hash(s, (0x123456 + l, 0xfedcba - l));
    h2 % m
}

fn main() {
    let strs = BufReader::new(io::stdin()).lines().map(|l| l.unwrap()).collect::<Vec<_>>();

    //let bits = 32 - (strs.len() as u32).leading_zeros();
    //let size = 1 << bits;
    // 90% load factor on the main table (1.11 = 1 / 0.9)
    let size = strs.len() as u64 * 111 / 100;
    let size_bits = 32 - (size as u32).leading_zeros();
    let raw_size = 1 << size_bits;

    // Prepare `r` buckets
    // 400% load factor on the intermediate table.
    let r = cmp::max(1, raw_size / 4);
    //let r = strs.len() as u64 / 4;
    let mut buckets = Vec::with_capacity(r as usize);
    for _ in 0 .. r {
        buckets.push(Vec::new());
    }

    // Use hash function `g` to hash items into buckets
    //let k_g = rand::random();
    let k_g = (0x123456, 0xfedcba);
    for (i, s) in strs.iter().enumerate() {
        let h = hash(s, k_g);
        buckets[(h % r) as usize].push(i);
    }


    // Sort buckets by size
    let mut order = (0 .. r as usize).collect::<Vec<_>>();
    order.sort_by_key(|&i| buckets[i].len());
    let order = order;


    // Process each bucket
    let kphf = 1;
    let mut ls = iter::repeat(0).take(r as usize).collect::<Vec<_>>();
    let mut t = iter::repeat(0).take(size as usize).collect::<Vec<_>>();
    for i in order.into_iter().rev() {
        'a: for l in 0.. {
            // Try hashing each string in `buckets[i]` using phi_l
            let k = (k_g.0 + l, k_g.1 - l);

            // Make sure phi_l maps all elements of `bucket[i]` to distinct locations.
            let mut tmp = iter::repeat(0).take(size as usize).collect::<Vec<_>>();
            for &idx in &buckets[i] {
                let slot = hash(&strs[idx], k) % raw_size;
                if slot >= size || tmp[slot as usize] >= kphf {
                    continue 'a;
                } else {
                    tmp[slot as usize] += 1;
                }
            }

            // Make sure `phi_l` maps all elements to locations that aren't yet used in `t`.
            for j in 0 .. size {
                if tmp[j as usize] + t[j as usize] > kphf {
                    continue 'a;
                }
            }

            // Success!
            ls[i] = l;
            for j in 0 .. size {
                t[j as usize] += tmp[j as usize];
            }
            break;
        }
    }

    println!("sizes {} {} {}", size, raw_size, r);

    print!("hashes");
    for s in &strs {
        print!(" {}", phf(s, &ls, r, raw_size));
    }
    println!("");

    print!("params");
    for &l in &ls {
        print!(" {}", l);
    }
    println!("");
}
