use gtest::{Program, System};
use pebbles_game_io::*;

const USER: u64 = 42;
const GAS: u128 = 4200000000;

#[test]
fn test_user_turn() {
    let sys = System::new();
    sys.init_logger();
    let program = Program::current(&sys);
    sys.mint_to(USER, GAS * 10000000000);
    // Initialize the game
    let init_data = PebblesInit {
        difficulty: DifficultyLevel::Easy,
        pebbles_count: 10,
        max_pebbles_per_turn: 3,
    };
    program.send(USER, init_data);
    sys.run_next_block();
    // User takes a turn by removing 2 pebbles
    let action = PebblesAction::Turn(2);
    program.send(USER, action);
    sys.run_next_block();

    // Check the game state to ensure pebbles were removed
    let state: GameState = program.read_state("state").expect("Failed to get state");
    assert_eq!(
        state.pebbles_remaining, 7,
        "Pebbles count did not update correctly"
    );
}

#[test]
fn test_program_turn_and_winning() {
    let sys = System::new();
    sys.init_logger();
    let program = Program::current(&sys);
    sys.mint_to(USER, GAS * 10000000000);

    // Initialize with just 1 pebble to check if Program wins immediately when it goes first
    let init_data = PebblesInit {
        difficulty: DifficultyLevel::Hard,
        pebbles_count: 1,
        max_pebbles_per_turn: 3,
    };

    program.send(USER, init_data);
    sys.run_next_block();
    let action = PebblesAction::Turn(3);
    program.send(USER, action);
    sys.run_next_block();
    // Check if the game is already won by the Program
    let state: GameState = program.read_state("state").expect("Failed to get state");
    assert_eq!(
        state.winner,
        Some(Player::User),
        "Program should win immediately"
    );
}

#[test]
fn test_give_up() {
    let sys = System::new();
    sys.init_logger();
    let program = Program::current(&sys);
    sys.mint_to(USER, GAS * 10000000000);

    // Initialize the game
    let init_data = PebblesInit {
        difficulty: DifficultyLevel::Easy,
        pebbles_count: 10,
        max_pebbles_per_turn: 3,
    };
    program.send(USER, init_data);
    sys.run_next_block();

    // User decides to give up
    let action = PebblesAction::GiveUp;
    program.send(USER, action);
    sys.run_next_block();

    // Check that Program is the winner
    let state: GameState = program.read_state("state").expect("Failed to get state");
    assert_eq!(
        state.winner,
        Some(Player::Program),
        "Program should be the winner after user gives up"
    );
}

#[test]
fn test_restart() {
    let sys = System::new();
    sys.init_logger();
    let program = Program::current(&sys);
    sys.mint_to(USER, GAS * 10000000000);

    // Initialize and play a bit
    let init_data = PebblesInit {
        difficulty: DifficultyLevel::Easy,
        pebbles_count: 15,
        max_pebbles_per_turn: 3,
    };
    program.send(USER, init_data);
    sys.run_next_block();

    // User removes 3 pebbles
    let action = PebblesAction::Turn(3);
    program.send(USER, action);
    sys.run_next_block();

    // Restart the game with new settings
    let restart_action = PebblesAction::Restart {
        difficulty: DifficultyLevel::Hard,
        pebbles_count: 20,
        max_pebbles_per_turn: 4,
    };
    program.send(USER, restart_action);
    sys.run_next_block();

    // Check that game state has been reset
    let state: GameState = program.read_state("state").expect("Failed to get state");
    assert_eq!(
        state.pebbles_remaining, 15,
        "Pebbles count did not reset correctly"
    );
    assert_eq!(
        state.max_pebbles_per_turn, 4,
        "Max pebbles per turn did not reset correctly"
    );
    assert_eq!(
        state.difficulty,
        DifficultyLevel::Hard,
        "Difficulty level did not reset correctly"
    );
    assert!(
        state.winner.is_none(),
        "Winner should be none after restart"
    );
}
