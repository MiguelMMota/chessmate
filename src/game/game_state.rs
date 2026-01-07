use godot::prelude::*;
use super::board::{Board, GameStatus};
use super::piece::{Position, Move, PieceType};
use super::rules::{generate_legal_moves, get_game_status};

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

    /// Check if moving the selected piece to the given position is a promotion
    /// Returns true if the move would be a pawn promotion
    #[func]
    pub fn is_promotion_move(&self, row: i32, col: i32) -> bool {
        let to = Position::new(row as i8, col as i8);

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
    /// piece_type: "queen", "rook", "bishop", or "knight"
    /// Returns true if the move was successful, false otherwise
    #[func]
    pub fn try_move_selected_with_promotion(&mut self, row: i32, col: i32, piece_type: GString) -> bool {
        let to = Position::new(row as i8, col as i8);

        if let Some(from) = self.selected_position {
            let legal_moves = generate_legal_moves(&self.board, from);

            let promotion_piece = match piece_type.to_string().to_lowercase().as_str() {
                "queen" => PieceType::Queen,
                "rook" => PieceType::Rook,
                "bishop" => PieceType::Bishop,
                "knight" => PieceType::Knight,
                _ => return false,
            };

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
    /// Returns: "ongoing", "check", "checkmate_white", "checkmate_black", "stalemate", "draw"
    #[func]
    pub fn get_game_status(&self) -> GString {
        match get_game_status(&self.board) {
            GameStatus::Ongoing => "ongoing".into(),
            GameStatus::Check => "check".into(),
            GameStatus::Checkmate(color) => match color {
                super::piece::Color::White => "checkmate_white".into(),
                super::piece::Color::Black => "checkmate_black".into(),
            },
            GameStatus::Stalemate => "stalemate".into(),
            GameStatus::DrawInsufficientMaterial => "draw".into(),
        }
    }

    /// Check if game is over
    #[func]
    pub fn is_game_over(&self) -> bool {
        let status = get_game_status(&self.board);
        !matches!(status, GameStatus::Ongoing | GameStatus::Check)
    }
}
