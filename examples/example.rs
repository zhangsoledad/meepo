extern crate meepo;
extern crate tiny_keccak;

use meepo::meepo;
use std::{thread, time};

fn main() {
    let now = time::Instant::now();
    let fab30 = factorial(30);
    println!("fab30 {:?} elapsed {:?}", fab30, now.elapsed());

    let now = time::Instant::now();
    let fab30 = factorial(30);
    println!("fab30 {:?} cached elapsed {:?}", fab30, now.elapsed());

    let now = time::Instant::now();
    let s = swl(100);
    println!("swl {:?} elapsed {:?}", s, now.elapsed());

    let now = time::Instant::now();
    let s = swl(100);
    println!("swl {:?} cached elapsed {:?}", s, now.elapsed());

}

#[meepo(maxsize = 100)]
fn factorial(mut n: u128) -> u128 {
    let mut p = 1;
    while n > 1 {
        p *= n;
        n -= 1;
    }
    p
}


#[meepo(maxsize = 100)]
fn swl(n: u128) -> u128 {
    thread::sleep(time::Duration::from_secs(1));
    n
}
