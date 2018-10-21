extern crate meepo;
extern crate tiny_keccak;

use meepo::meepo;
use std::{thread, time};

fn main() {
    let now = time::Instant::now();
    let fab30 = factorial(30);
    println!("fab20 elapsed {:?}", now.elapsed());
    // assert_eq!(fab20, 2432902008176640000);
    let now = time::Instant::now();
    let fab30 = factorial(30);
    println!("fab20 cached elapsed {:?}", now.elapsed());

    let now = time::Instant::now();
    let _ = swl(100);
    println!("swl excute elapsed {:?}", now.elapsed());

    let now = time::Instant::now();
    let _ = swl(100);
    println!("swl cached elapsed {:?}", now.elapsed());

}

#[meepo(maxsize = 100)]
fn factorial(n: u128) -> u128 {
    let mut ret = n;
    let mut p = 1;
    while ret > 1 {
        p *= ret;
        ret -= 1;
    }
    p
}


#[meepo(maxsize = 100)]
fn swl(n: u128) -> u128 {
    thread::sleep(time::Duration::from_secs(1));
    n
}
