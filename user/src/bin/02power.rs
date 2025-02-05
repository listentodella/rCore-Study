#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;

const SIZE: usize = 10;
const P: u32 = 3;
const STEP: usize = 100_000;
const MOD: u32 = 10_007;

#[no_mangle]
fn main() -> i32 {
    let mut pow = [0u32; SIZE];
    let mut index = 0usize;
    pow[index] = 1;

    for i in 1..=STEP {
        let last = pow[index];
        index = (index + 1) % SIZE;
        pow[index] = (last * P) % MOD;

        if i % 10000 == 0 {
            println!("{} ^ {} = {}(MOD {})", P, i, pow[index], MOD);
        }
    }

    println!("Test Power Ok!");

    0
}
