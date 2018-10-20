extern crate meepo;
extern crate tiny_keccak;

use meepo::meepo;

fn main() {
    let _ = factorial(20);
}

#[meepo(maxsize = 100)]
fn factorial(n: u64) -> u64 {
    let mut ret = n;
    let mut p = 1;
    while ret > 1 {
        p *= ret;
        ret -= 1;
    }
    p
}
