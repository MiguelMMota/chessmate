use super::board::{Board, GameStatus};
use super::chess_clock::ChessClockSettings;
use super::piece::{Color, Move, PieceType, Position};
use super::rules::{generate_legal_moves, get_game_status};
use crate::ai::simple_opponent::select_weighted_move;
use std::collections::HashMap;

/// Pure Rust game state - no Godot dependencies
#[derive(Debug)]
pub struct ChessGame {
    board: Board,
    selected_position: Option<Position>,
}

impl ChessGame {
    /// Create a new chess game
    pub fn new() -> Self {
        Self {
            board: Board::new(),
            selected_position: None,
        }
    }
    /// Reset the game to initial position
    pub fn reset_game(&mut self) {
        self.board = Board::new();
        self.selected_position = None;
    }

    /// Reset the game with a chess clock
    /// initial_time_seconds: time for each player in seconds
    /// increment_seconds: time added after each move in seconds
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
    pub fn get_piece_at(&self, row: i8, col: i8) -> String {
        let pos = Position::new(row, col);
        if let Some(piece) = self.board.get_piece(pos) {
            piece.to_symbol().to_string()
        } else {
            String::new()
        }
    }

    /// Get the color of the piece at a position
    pub fn get_piece_color_at(&self, row: i8, col: i8) -> Option<Color> {
        let pos = Position::new(row, col);
        self.board.get_piece(pos).map(|piece| piece.color)
    }

    /// Get whose turn it is
    pub fn get_current_turn(&self) -> Color {
        self.board.current_turn()
    }

    /// Try to select a piece at the given position
    /// Returns true if a piece was selected, false otherwise
    pub fn select_piece(&mut self, row: i8, col: i8) -> bool {
        let pos = Position::new(row, col);

        if let Some(piece) = self.board.get_piece(pos) {
            if piece.color == self.board.current_turn() {
                self.selected_position = Some(pos);
                return true;
            }
        }

        false
    }

    /// Get legal moves for the currently selected piece
    /// Returns a vector of move positions
    pub fn get_legal_moves_for_selected(&self) -> Vec<Position> {
        if let Some(from) = self.selected_position {
            let moves = generate_legal_moves(&self.board, from);
            moves.into_iter().map(|mv| mv.to).collect()
        } else {
            Vec::new()
        }
    }

    /// Check if moving the selected piece to the given position is a promotion
    /// Returns true if the move would be a pawn promotion
    pub fn is_promotion_move(&self, row: i8, col: i8) -> bool {
        let to = Position::new(row, col);

        if let Some(from) = self.selected_position {
            let legal_moves = generate_legal_moves(&self.board, from);

            for mv in legal_moves {
                if mv.to == to && mv.promotion.is_some() {
                    return true;
                }
            }
        }

        false
    }

    /// Try to move the selected piece to the given position with a specific promotion piece
    /// Returns true if the move was successful, false otherwise
    pub fn try_move_selected_with_promotion(
        &mut self,
        row: i8,
        col: i8,
        promotion_piece: PieceType,
    ) -> bool {
        let to = Position::new(row, col);

        if let Some(from) = self.selected_position {
            let legal_moves = generate_legal_moves(&self.board, from);

            // Check if this is a legal move
            for mv in legal_moves {
                if mv.to == to {
                    let final_move = if mv.promotion.is_some() {
                        Move::with_promotion(from, to, promotion_piece)
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

    /// Try to move the selected piece to the given position
    /// Returns true if the move was successful, false otherwise
    /// NOTE: This defaults to Queen for promotions - use try_move_selected_with_promotion for other pieces
    pub fn try_move_selected(&mut self, row: i8, col: i8) -> bool {
        let to = Position::new(row, col);

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
    pub fn deselect_piece(&mut self) {
        self.selected_position = None;
    }

    /// Get the selected position or None if nothing selected
    pub fn get_selected_position(&self) -> Option<Position> {
        self.selected_position
    }

    /// Get the current game status
    pub fn get_game_status(&self) -> GameStatus {
        // First check for time loss
        if let Some(color) = self.board.check_time_loss() {
            return GameStatus::TimeLoss(color);
        }

        // Then check regular game status
        get_game_status(&self.board)
    }

    /// Check if game is over
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
    pub fn tick_clock(&mut self) -> bool {
        self.board.tick_clock()
    }

    /// Get remaining time for a color in seconds (None if no clock)
    pub fn get_remaining_time(&self, color: Color) -> Option<i32> {
        self.board.get_remaining_time(color)
    }

    /// Check if the game has a chess clock enabled
    pub fn has_clock(&self) -> bool {
        self.board.has_clock()
    }

    /// Make an AI move for the current player
    /// Returns true if a move was made, false if no legal moves available
    pub fn make_ai_move(&mut self) -> bool {
        if let Some(mv) = select_weighted_move(&self.board) {
            self.board.make_move(mv);
            self.selected_position = None;
            true
        } else {
            false
        }
    }

    /// Get a reference to the internal board (for server/network use)
    pub fn board(&self) -> &Board {
        &self.board
    }

    /// Get the board squares as a 2D array (for serialization)
    pub fn board_squares(&self) -> [[Option<super::piece::Piece>; 8]; 8] {
        let mut squares = [[None; 8]; 8];
        for row in 0..8 {
            for col in 0..8 {
                let pos = Position::new(row, col);
                squares[row as usize][col as usize] = self.board.get_piece(pos);
            }
        }
        squares
    }
}
