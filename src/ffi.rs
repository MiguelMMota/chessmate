// FFI layer for communicating with external clients
// This layer should be thin and performant

use crate::game::game_state::ChessGame;
use crate::game::board::GameStatus;
use crate::game::piece::{Color, PieceType};
use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::ptr;
use std::collections::HashMap;
use std::sync::{Mutex, LazyLock};

// Global game state storage
// In a real implementation, you'd manage multiple game instances
static GAME_INSTANCES: LazyLock<Mutex<HashMap<u32, ChessGame>>> = LazyLock::new(|| Mutex::new(HashMap::new()));
static NEXT_GAME_ID: LazyLock<Mutex<u32>> = LazyLock::new(|| Mutex::new(0));

#[repr(C)]
pub struct GameState {
    pub game_id: u32,
    pub current_turn: u8,  // 0 = White, 1 = Black
    pub status: u8,  // 0 = Ongoing, 1 = Check, 2 = Checkmate White, 3 = Checkmate Black, 4 = Stalemate, 5 = Draw, 6 = TimeLoss White, 7 = TimeLoss Black
    pub white_time: i32,  // -1 if no clock
    pub black_time: i32,  // -1 if no clock
    pub board_state: *mut c_char,  // JSON representation of board state
}

#[repr(C)]
pub struct ActionResult {
    pub success: bool,
    pub game_state: GameState,
    pub error_message: *mut c_char,
}

/// Initialize a new game
/// Returns game_id
#[no_mangle]
pub extern "C" fn initialize_game(initial_time_seconds: i32, increment_seconds: i32) -> u32 {
    let mut game = ChessGame::new();

    if initial_time_seconds > 0 {
        game.reset_game_with_clock(initial_time_seconds, increment_seconds);
    }

    let mut instances = GAME_INSTANCES.lock().unwrap();
    let mut next_id = NEXT_GAME_ID.lock().unwrap();
    let game_id = *next_id;
    *next_id += 1;

    instances.insert(game_id, game);
    game_id
}

/// Process an action and return the new game state
/// action_type: 0 = MovePiece
/// data: JSON string with action data
#[no_mangle]
pub extern "C" fn process_action(game_id: u32, action_type: u8, data: *const c_char) -> ActionResult {
    let mut instances = GAME_INSTANCES.lock().unwrap();

    let game = match instances.get_mut(&game_id) {
        Some(g) => g,
        None => {
            return ActionResult {
                success: false,
                game_state: get_empty_game_state(),
                error_message: create_c_string("Invalid game_id"),
            };
        }
    };

    match action_type {
        0 => {
            // MovePiece action
            let data_str = unsafe {
                match CStr::from_ptr(data).to_str() {
                    Ok(s) => s,
                    Err(_) => {
                        return ActionResult {
                            success: false,
                            game_state: get_game_state_from_game(game_id, game),
                            error_message: create_c_string("Invalid UTF-8 in data"),
                        };
                    }
                }
            };

            // Parse data as JSON: {"from": {"row": 0, "col": 0}, "to": {"row": 1, "col": 0}, "promotion": "queen"}
            // For simplicity in this implementation, we'll use a simple format: "from_row,from_col,to_row,to_col" or "from_row,from_col,to_row,to_col,promotion"
            let parts: Vec<&str> = data_str.split(',').collect();
            if parts.len() < 4 {
                return ActionResult {
                    success: false,
                    game_state: get_game_state_from_game(game_id, game),
                    error_message: create_c_string("Invalid move data format"),
                };
            }

            let from_row: i8 = match parts[0].parse() {
                Ok(v) => v,
                Err(_) => {
                    return ActionResult {
                        success: false,
                        game_state: get_game_state_from_game(game_id, game),
                        error_message: create_c_string("Invalid from_row"),
                    };
                }
            };

            let from_col: i8 = match parts[1].parse() {
                Ok(v) => v,
                Err(_) => {
                    return ActionResult {
                        success: false,
                        game_state: get_game_state_from_game(game_id, game),
                        error_message: create_c_string("Invalid from_col"),
                    };
                }
            };

            let to_row: i8 = match parts[2].parse() {
                Ok(v) => v,
                Err(_) => {
                    return ActionResult {
                        success: false,
                        game_state: get_game_state_from_game(game_id, game),
                        error_message: create_c_string("Invalid to_row"),
                    };
                }
            };

            let to_col: i8 = match parts[3].parse() {
                Ok(v) => v,
                Err(_) => {
                    return ActionResult {
                        success: false,
                        game_state: get_game_state_from_game(game_id, game),
                        error_message: create_c_string("Invalid to_col"),
                    };
                }
            };

            // Select the piece first
            if !game.select_piece(from_row, from_col) {
                return ActionResult {
                    success: false,
                    game_state: get_game_state_from_game(game_id, game),
                    error_message: create_c_string("Cannot select piece at from position"),
                };
            }

            // Check if promotion
            let success = if parts.len() > 4 {
                let promotion_piece = match parts[4] {
                    "queen" => PieceType::Queen,
                    "rook" => PieceType::Rook,
                    "bishop" => PieceType::Bishop,
                    "knight" => PieceType::Knight,
                    _ => {
                        return ActionResult {
                            success: false,
                            game_state: get_game_state_from_game(game_id, game),
                            error_message: create_c_string("Invalid promotion piece"),
                        };
                    }
                };
                game.try_move_selected_with_promotion(to_row, to_col, promotion_piece)
            } else {
                game.try_move_selected(to_row, to_col)
            };

            ActionResult {
                success,
                game_state: get_game_state_from_game(game_id, game),
                error_message: if success { ptr::null_mut() } else { create_c_string("Invalid move") },
            }
        }
        _ => ActionResult {
            success: false,
            game_state: get_game_state_from_game(game_id, game),
            error_message: create_c_string("Unknown action type"),
        },
    }
}

/// Get the current game state
#[no_mangle]
pub extern "C" fn get_game_state(game_id: u32) -> GameState {
    let instances = GAME_INSTANCES.lock().unwrap();

    match instances.get(&game_id) {
        Some(game) => get_game_state_from_game(game_id, game),
        None => get_empty_game_state(),
    }
}

/// Tick the game clock
#[no_mangle]
pub extern "C" fn tick_clock(game_id: u32) -> bool {
    let mut instances = GAME_INSTANCES.lock().unwrap();

    match instances.get_mut(&game_id) {
        Some(game) => game.tick_clock(),
        None => false,
    }
}

/// Make an AI move
#[no_mangle]
pub extern "C" fn make_ai_move(game_id: u32) -> ActionResult {
    let mut instances = GAME_INSTANCES.lock().unwrap();

    let game = match instances.get_mut(&game_id) {
        Some(g) => g,
        None => {
            return ActionResult {
                success: false,
                game_state: get_empty_game_state(),
                error_message: create_c_string("Invalid game_id"),
            };
        }
    };

    let success = game.make_ai_move();

    ActionResult {
        success,
        game_state: get_game_state_from_game(game_id, game),
        error_message: if success { ptr::null_mut() } else { create_c_string("No legal moves available") },
    }
}

/// Free a game instance
#[no_mangle]
pub extern "C" fn free_game(game_id: u32) {
    let mut instances = GAME_INSTANCES.lock().unwrap();
    instances.remove(&game_id);
}

/// Free a C string returned by the FFI
#[no_mangle]
pub extern "C" fn free_string(s: *mut c_char) {
    if !s.is_null() {
        unsafe {
            let _ = CString::from_raw(s);
        }
    }
}

// Helper functions

fn get_game_state_from_game(game_id: u32, game: &ChessGame) -> GameState {
    let status = game.get_game_status();
    let status_code = match status {
        GameStatus::Ongoing => 0,
        GameStatus::Check => 1,
        GameStatus::Checkmate(Color::White) => 2,
        GameStatus::Checkmate(Color::Black) => 3,
        GameStatus::Stalemate => 4,
        GameStatus::DrawInsufficientMaterial => 5,
        GameStatus::TimeLoss(Color::White) => 6,
        GameStatus::TimeLoss(Color::Black) => 7,
    };

    let current_turn = match game.get_current_turn() {
        Color::White => 0,
        Color::Black => 1,
    };

    let white_time = game.get_remaining_time(Color::White).unwrap_or(-1);
    let black_time = game.get_remaining_time(Color::Black).unwrap_or(-1);

    // Serialize board state to JSON
    // For now, we'll create a simple representation
    let mut board_json = String::from("{\"pieces\":[");
    let mut pieces = Vec::new();

    for row in 0..8 {
        for col in 0..8 {
            let piece = game.get_piece_at(row, col);
            if !piece.is_empty() {
                let color = game.get_piece_color_at(row, col);
                let color_str = match color {
                    Some(Color::White) => "white",
                    Some(Color::Black) => "black",
                    None => continue,
                };
                pieces.push(format!("{{\"row\":{},\"col\":{},\"piece\":\"{}\",\"color\":\"{}\"}}", row, col, piece, color_str));
            }
        }
    }

    board_json.push_str(&pieces.join(","));
    board_json.push_str("]}");

    GameState {
        game_id,
        current_turn,
        status: status_code,
        white_time,
        black_time,
        board_state: create_c_string(&board_json),
    }
}

fn get_empty_game_state() -> GameState {
    GameState {
        game_id: 0,
        current_turn: 0,
        status: 0,
        white_time: -1,
        black_time: -1,
        board_state: create_c_string("{}"),
    }
}

fn create_c_string(s: &str) -> *mut c_char {
    match CString::new(s) {
        Ok(cs) => cs.into_raw(),
        Err(_) => ptr::null_mut(),
    }
}
