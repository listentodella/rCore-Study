#![no_std]
#![no_main]

use user_lib::{get_time, println, yield_};

#[no_mangle]
fn main() -> i32 {
    println!("Test sleep Start!");
    let current_timer = get_time();
    let wait_for = current_timer + 3000;
    while get_time() < wait_for {
        yield_();
    }

    println!("Test sleep Done!");
    0
}
