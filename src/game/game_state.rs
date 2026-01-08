use godot::prelude::*;
use super::board::{Board, GameStatus};
use super::piece::{Position, Move, PieceType, Color};
use super::rules::{generate_legal_moves, get_game_status};
use super::chess_clock::ChessClockSettings;
use crate::ai::simple_opponent::select_weighted_move;
use std::collections::HashMap;

#[derive(GodotClass)]
#[class(base=Node)]
pub struct ChessGame {
    board: Board,
    selected_position: Option<Position>,

    #[base]
    base: Base<Node>,
}

#[godot_api]
impl INode for ChessGame {
    fn init(base: Base<Node>) -> Self {
        Self {
            board: Board::new(),
            selected_position: None,
            base,
        }
    }
}

#[godot_api]
impl ChessGame {
    /// Reset the game to initial position
    #[func]
    pub fn reset_game(&mut self) {
        self.board = Board::new();
        self.selected_position = None;
    }

    /// Reset the game with a chess clock
    /// initial_time_seconds: time for each player in seconds
    /// increment_seconds: time added after each move in seconds
    #[func]
    pub fn reset_game_with_clock(&mut self, initial_time_seconds: i32, increment_seconds: i32) {
        let mut initial_times = HashMap::new();
        initial_times.insert(0, initial_time_seconds); // White
        initial_times.insert(1, initial_time_seconds); // Black

        let mut increments = HashMap::new();
        increments.insert(0, increment_seconds); // White
        increments.insert(1, increment_seconds); // Black

        let clock_settings = ChessClockSettings {
            initial_times,
            move_increments: increments,
            triggers: vec![],
        };

        self.board = Board::new_with_clock(Some(clock_settings));
        self.selected_position = None;
    }

    /// Get the piece at a position (returns symbol as String, empty if no piece)
    #[func]
    pub fn get_piece_at(&self, row: i32, col: i32) -> GString {
        let pos = Position::new(row as i8, col as i8);
        if let Some(piece) = self.board.get_piece(pos) {
            GString::from(&piece.to_symbol().to_string())
        } else {
            GString::new()
        }
    }

    /// Get the color of the piece at a position ("white", "black", or "" if no piece)
    #[func]
    pub fn get_piece_color_at(&self, row: i32, col: i32) -> GString {
        let pos = Position::new(row as i8, col as i8);
        if let Some(piece) = self.board.get_piece(pos) {
            match piece.color {
                super::piece::Color::White => "white".into(),
                super::piece::Color::Black => "black".into(),
            }
        } else {
            GString::new()
        }
    }

    /// Get whose turn it is ("white" or "black")
    #[func]
    pub fn get_current_turn(&self) -> GString {
        match self.board.current_turn() {
            super::piece::Color::White => "white".into(),
            super::piece::Color::Black => "black".into(),
        }
    }

    /// Try to select a piece at the given position
    /// Returns true if a piece was selected, false otherwise
    #[func]
    pub fn select_piece(&mut self, row: i32, col: i32) -> bool {
        let pos = Position::new(row as i8, col as i8);

        if let Some(piece) = self.board.get_piece(pos) {
            if piece.color == self.board.current_turn() {
                self.selected_position = Some(pos);
                return true;
            }
        }

        false
    }

    /// Get legal moves for the currently selected piece
    /// Returns an array of positions as [row, col, row, col, ...]
    #[func]
    pub fn get_legal_moves_for_selected(&self) -> PackedInt32Array {
        let mut result = PackedInt32Array::new();

        if let Some(from) = self.selected_position {
            let moves = generate_legal_moves(&self.board, from);
            for mv in moves {
                result.push(mv.to.row as i32);
                result.push(mv.to.col as i32);
            }
        }

        result
    }

    /// Try to move the selected piece to the given position
    /// Returns true if the move was successful, false otherwise
    #[func]
    pub fn try_move_selected(&mut self, row: i32, col: i32) -> bool {
        let to = Position::new(row as i8, col as i8);

        if let Some(from) = self.selected_position {
            let legal_moves = generate_legal_moves(&self.board, from);

            // Check if this is a legal move
            for mv in legal_moves {
                if mv.to == to {
                    // Handle pawn promotion - default to queen for now
                    let final_move = if mv.promotion.is_some() {
                        Move::with_promotion(from, to, PieceType::Queen)
                    } else {
                        mv
                    };

                    self.board.make_move(final_move);
                    self.selected_position = None;
                    return true;
                }
            }
        }

        false
    }

    /// Deselect the currently selected piece
    #[func]
    pub fn deselect_piece(&mut self) {
        self.selected_position = None;
    }

    /// Get the selected position as [row, col] or empty array if nothing selected
    #[func]
    pub fn get_selected_position(&self) -> PackedInt32Array {
        let mut result = PackedInt32Array::new();
        if let Some(pos) = self.selected_position {
            result.push(pos.row as i32);
            result.push(pos.col as i32);
        }
        result
    }

    /// Get the current game status
    /// Returns: "ongoing", "check", "checkmate_white", "checkmate_black", "stalemate", "draw", "timeloss_white", "timeloss_black"
    #[func]
    pub fn get_game_status(&self) -> GString {
        // First check for time loss
        if let Some(color) = self.board.check_time_loss() {
            return match color {
                Color::White => "timeloss_white".into(),
                Color::Black => "timeloss_black".into(),
            };
        }

        // Then check regular game status
        match get_game_status(&self.board) {
            GameStatus::Ongoing => "ongoing".into(),
            GameStatus::Check => "check".into(),
            GameStatus::Checkmate(color) => match color {
                Color::White => "checkmate_white".into(),
                Color::Black => "checkmate_black".into(),
            },
            GameStatus::Stalemate => "stalemate".into(),
            GameStatus::DrawInsufficientMaterial => "draw".into(),
            GameStatus::TimeLoss(color) => match color {
                Color::White => "timeloss_white".into(),
                Color::Black => "timeloss_black".into(),
            },
        }
    }

    /// Check if game is over
    #[func]
    pub fn is_game_over(&self) -> bool {
        // Check time loss first
        if self.board.check_time_loss().is_some() {
            return true;
        }

        let status = get_game_status(&self.board);
        !matches!(status, GameStatus::Ongoing | GameStatus::Check)
    }

    /// Tick the chess clock (should be called every second)
    /// Returns false if the active player ran out of time
    #[func]
    pub fn tick_clock(&mut self) -> bool {
        self.board.tick_clock()
    }

    /// Get remaining time for White in seconds (-1 if no clock)
    #[func]
    pub fn get_white_time(&self) -> i32 {
        self.board.get_remaining_time(Color::White).unwrap_or(-1)
    }

    /// Get remaining time for Black in seconds (-1 if no clock)
    #[func]
    pub fn get_black_time(&self) -> i32 {
        self.board.get_remaining_time(Color::Black).unwrap_or(-1)
    }

    /// Check if the game has a chess clock enabled
    #[func]
    pub fn has_clock(&self) -> bool {
        self.board.has_clock()
    }

    /// Make an AI move for the current player
    /// Returns true if a move was made, false if no legal moves available
    #[func]
    pub fn make_ai_move(&mut self) -> bool {
        if let Some(mv) = select_weighted_move(&self.board) {
            self.board.make_move(mv);
            self.selected_position = None;
            true
        } else {
            false
        }
    }
}
