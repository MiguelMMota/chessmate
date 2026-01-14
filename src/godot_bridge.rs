// Godot-specific bridge - wraps the pure Rust game logic for Godot
// This is the ONLY file that should have Godot dependencies

use crate::game::board::GameStatus;
use crate::game::game_state::ChessGame as RustChessGame;
use crate::game::piece::{Color, PieceType};
use godot::prelude::*;

// Main extension struct for Godot
struct ChessMateExtension;

#[gdextension]
unsafe impl ExtensionLibrary for ChessMateExtension {}

/// Godot wrapper around the pure Rust ChessGame
#[derive(GodotClass)]
#[class(base=Node)]
pub struct ChessGame {
    game: RustChessGame,

    #[base]
    base: Base<Node>,
}

#[godot_api]
impl INode for ChessGame {
    fn init(base: Base<Node>) -> Self {
        Self {
            game: RustChessGame::new(),
            base,
        }
    }
}

#[godot_api]
impl ChessGame {
    /// Reset the game to initial position
    #[func]
    pub fn reset_game(&mut self) {
        self.game.reset_game();
    }

    /// Reset the game with a chess clock
    #[func]
    pub fn reset_game_with_clock(&mut self, initial_time_seconds: i32, increment_seconds: i32) {
        self.game
            .reset_game_with_clock(initial_time_seconds, increment_seconds);
    }

    /// Get the piece at a position
    #[func]
    pub fn get_piece_at(&self, row: i32, col: i32) -> GString {
        GString::from(&self.game.get_piece_at(row as i8, col as i8))
    }

    /// Get the color of the piece at a position
    #[func]
    pub fn get_piece_color_at(&self, row: i32, col: i32) -> GString {
        match self.game.get_piece_color_at(row as i8, col as i8) {
            Some(Color::White) => "white".into(),
            Some(Color::Black) => "black".into(),
            None => GString::new(),
        }
    }

    /// Get whose turn it is
    #[func]
    pub fn get_current_turn(&self) -> GString {
        match self.game.get_current_turn() {
            Color::White => "white".into(),
            Color::Black => "black".into(),
        }
    }

    /// Set whose turn it is (for network synchronization)
    #[func]
    pub fn set_current_turn(&mut self, color: GString) {
        let turn_color = match color.to_string().to_lowercase().as_str() {
            "white" => Color::White,
            "black" => Color::Black,
            _ => return,
        };
        self.game.set_current_turn(turn_color);
    }

    /// Try to select a piece at the given position
    #[func]
    pub fn select_piece(&mut self, row: i32, col: i32) -> bool {
        self.game.select_piece(row as i8, col as i8)
    }

    /// Get legal moves for the currently selected piece
    #[func]
    pub fn get_legal_moves_for_selected(&self) -> PackedInt32Array {
        let mut result = PackedInt32Array::new();
        let moves = self.game.get_legal_moves_for_selected();
        for pos in moves {
            result.push(pos.row as i32);
            result.push(pos.col as i32);
        }
        result
    }

    /// Check if moving the selected piece to the given position is a promotion
    #[func]
    pub fn is_promotion_move(&self, row: i32, col: i32) -> bool {
        self.game.is_promotion_move(row as i8, col as i8)
    }

    /// Try to move the selected piece to the given position with a specific promotion piece
    #[func]
    pub fn try_move_selected_with_promotion(
        &mut self,
        row: i32,
        col: i32,
        piece_type: GString,
    ) -> bool {
        let promotion_piece = match piece_type.to_string().to_lowercase().as_str() {
            "queen" => PieceType::Queen,
            "rook" => PieceType::Rook,
            "bishop" => PieceType::Bishop,
            "knight" => PieceType::Knight,
            _ => return false,
        };
        self.game
            .try_move_selected_with_promotion(row as i8, col as i8, promotion_piece)
    }

    /// Try to move the selected piece to the given position
    #[func]
    pub fn try_move_selected(&mut self, row: i32, col: i32) -> bool {
        self.game.try_move_selected(row as i8, col as i8)
    }

    /// Deselect the currently selected piece
    #[func]
    pub fn deselect_piece(&mut self) {
        self.game.deselect_piece();
    }

    /// Get the selected position
    #[func]
    pub fn get_selected_position(&self) -> PackedInt32Array {
        let mut result = PackedInt32Array::new();
        if let Some(pos) = self.game.get_selected_position() {
            result.push(pos.row as i32);
            result.push(pos.col as i32);
        }
        result
    }

    /// Get the current game status
    #[func]
    pub fn get_game_status(&self) -> GString {
        match self.game.get_game_status() {
            GameStatus::Ongoing => "ongoing".into(),
            GameStatus::Check => "check".into(),
            GameStatus::Checkmate(Color::White) => "checkmate_white".into(),
            GameStatus::Checkmate(Color::Black) => "checkmate_black".into(),
            GameStatus::Stalemate => "stalemate".into(),
            GameStatus::DrawInsufficientMaterial => "draw".into(),
            GameStatus::TimeLoss(Color::White) => "timeloss_white".into(),
            GameStatus::TimeLoss(Color::Black) => "timeloss_black".into(),
        }
    }

    /// Check if game is over
    #[func]
    pub fn is_game_over(&self) -> bool {
        self.game.is_game_over()
    }

    /// Tick the chess clock
    #[func]
    pub fn tick_clock(&mut self) -> bool {
        self.game.tick_clock()
    }

    /// Get remaining time for White in seconds
    #[func]
    pub fn get_white_time(&self) -> i32 {
        self.game.get_remaining_time(Color::White).unwrap_or(-1)
    }

    /// Get remaining time for Black in seconds
    #[func]
    pub fn get_black_time(&self) -> i32 {
        self.game.get_remaining_time(Color::Black).unwrap_or(-1)
    }

    /// Check if the game has a chess clock enabled
    #[func]
    pub fn has_clock(&self) -> bool {
        self.game.has_clock()
    }

    /// Make an AI move for the current player
    #[func]
    pub fn make_ai_move(&mut self) -> bool {
        self.game.make_ai_move()
    }

    /// Clear a square on the board (set to empty)
    #[func]
    pub fn clear_square(&mut self, row: i32, col: i32) {
        self.game.clear_square(row as i8, col as i8);
    }

    /// Clear the en passant target
    #[func]
    pub fn clear_en_passant_target(&mut self) {
        self.game.clear_en_passant_target();
    }

    /// Place a piece on the board at the given position
    #[func]
    pub fn place_piece(
        &mut self,
        row: i32,
        col: i32,
        piece_type: GString,
        color: GString,
        id: i32,
    ) -> bool {
        let ptype = match piece_type.to_string().to_lowercase().as_str() {
            "king" => PieceType::King,
            "queen" => PieceType::Queen,
            "rook" => PieceType::Rook,
            "bishop" => PieceType::Bishop,
            "knight" => PieceType::Knight,
            "pawn" => PieceType::Pawn,
            _ => return false,
        };

        let pcolor = match color.to_string().to_lowercase().as_str() {
            "white" => Color::White,
            "black" => Color::Black,
            _ => return false,
        };

        self.game.place_piece(row as i8, col as i8, ptype, pcolor, id as u8);
        true
    }

    /// Set white's remaining time (for clock synchronization)
    #[func]
    pub fn set_white_time(&mut self, seconds: i32) {
        self.game.set_white_time(seconds);
    }

    /// Set black's remaining time (for clock synchronization)
    #[func]
    pub fn set_black_time(&mut self, seconds: i32) {
        self.game.set_black_time(seconds);
    }
}
