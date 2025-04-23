#![no_std]
use gstd::{exec, msg, prelude::*};
use pebbles_game_io::*;

static mut PEBBLES_GAME: Option<GameState> = None;

fn get_random_u32() -> u32 {
    let salt = msg::id();
    let (hash, _) = exec::random(salt.into()).expect("Random call failed");
    u32::from_le_bytes([hash[0], hash[1], hash[2], hash[3]])
}

#[no_mangle]
extern "C" fn init() {

}



#[no_mangle]
extern "C" fn handle() {
//
}

#[no_mangle]
extern "C" fn state() {
    let game_state = unsafe {
        PEBBLES_GAME
            .as_ref()
            .expect("Unexpected error in getting state")
    };
    msg::reply(game_state, 0).expect("Failed to send game state");
}
