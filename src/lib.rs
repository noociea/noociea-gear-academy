#![no_std]
#![allow(static_mut_refs)]
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
    let init_data: PebblesInit = msg::load().expect("Failed to load init data");

    assert!(
        init_data.pebbles_count > 0,
        "Pebbles count must be positive"
    );
    assert!(
        init_data.max_pebbles_per_turn > 0,
        "Max pebbles per turn must be positive"
    );

    let first_player = if get_random_u32() % 2 == 0 {
        Player::User
    } else {
        Player::Program
    };

    let mut game_state = GameState {
        pebbles_count: init_data.pebbles_count,
        max_pebbles_per_turn: init_data.max_pebbles_per_turn,
        pebbles_remaining: init_data.pebbles_count,
        difficulty: init_data.difficulty,
        first_player: first_player.clone(),
        winner: None,
    };
    if first_player == Player::Program {
        program_turn(&mut game_state);
    }
    unsafe {
        PEBBLES_GAME = Some(game_state);
    }
}

fn program_turn(game_state: &mut GameState) {
    let pebbles_to_remove = match game_state.difficulty {
        DifficultyLevel::Easy => 1 + (get_random_u32() % game_state.max_pebbles_per_turn),
        DifficultyLevel::Hard => {
            let remainder = game_state.pebbles_remaining % (game_state.max_pebbles_per_turn + 1);
            if remainder == 0 {
                game_state.max_pebbles_per_turn + 1
            } else {
                remainder
            }
        }
    };

    game_state.pebbles_remaining = game_state
        .pebbles_remaining
        .saturating_sub(pebbles_to_remove);

    if game_state.pebbles_remaining == 0 {
        game_state.winner = Some(Player::Program);
        msg::reply(PebblesEvent::Won(Player::Program), 0).expect("Failed to send reply");
    } else {
        msg::reply(PebblesEvent::CounterTurn(pebbles_to_remove), 0).expect("Failed to send reply");
    }
}

#[no_mangle]
extern "C" fn handle() {
    let action: PebblesAction = msg::load().expect("Failed to load action");
    let game_state = unsafe { PEBBLES_GAME.as_mut().expect("Game not initialized") };

    match action {
        PebblesAction::Turn(pebbles) => {
            assert!(
                pebbles > 0 && pebbles <= game_state.max_pebbles_per_turn,
                "Invalid move"
            );
            game_state.pebbles_remaining = game_state.pebbles_remaining.saturating_sub(pebbles);
            if game_state.pebbles_remaining == 0 {
                game_state.winner = Some(Player::User);
                msg::reply(PebblesEvent::Won(Player::User), 0).expect("Failed to send reply");
            } else {
                program_turn(game_state);
            }
        }
        PebblesAction::GiveUp => {
            game_state.winner = Some(Player::Program);
            msg::reply(PebblesEvent::Won(Player::Program), 0).expect("Failed to send reply");
        }
        PebblesAction::Restart {
            difficulty,
            pebbles_count,
            max_pebbles_per_turn,
        } => {
            *game_state = GameState {
                pebbles_count,
                max_pebbles_per_turn,
                pebbles_remaining: pebbles_count,
                difficulty,
                first_player: if get_random_u32() % 2 == 0 {
                    Player::User
                } else {
                    Player::Program
                },
                winner: None,
            };
            if game_state.first_player == Player::Program {
                program_turn(game_state);
            }
        }
    }
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
