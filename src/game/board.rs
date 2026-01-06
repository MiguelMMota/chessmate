use super::piece::{Color, Piece, PieceType, Position, Move};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameStatus {
    Ongoing,
    Check,
    Checkmate(Color), // Winner
    Stalemate,
    DrawInsufficientMaterial,
}

#[derive(Debug, Clone)]
pub struct CastlingRights {
    pub white_kingside: bool,
    pub white_queenside: bool,
    pub black_kingside: bool,
    pub black_queenside: bool,
}

impl CastlingRights {
    pub fn new() -> Self {
        Self {
            white_kingside: true,
            white_queenside: true,
            black_kingside: true,
            black_queenside: true,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Board {
    squares: [[Option<Piece>; 8]; 8],
    current_turn: Color,
    castling_rights: CastlingRights,
    en_passant_target: Option<Position>,
    halfmove_clock: u32,
    fullmove_number: u32,
}

impl Board {
    pub fn new() -> Self {
        let mut board = Self {
            squares: [[None; 8]; 8],
            current_turn: Color::White,
            castling_rights: CastlingRights::new(),
            en_passant_target: None,
            halfmove_clock: 0,
            fullmove_number: 1,
        };
        board.setup_initial_position();
        board
    }

    pub fn setup_initial_position(&mut self) {
        // Clear the board
        self.squares = [[None; 8]; 8];

        // Setup pawns
        for col in 0..8 {
            self.squares[1][col] = Some(Piece::new(PieceType::Pawn, Color::White));
            self.squares[6][col] = Some(Piece::new(PieceType::Pawn, Color::Black));
        }

        // Setup white pieces
        self.squares[0][0] = Some(Piece::new(PieceType::Rook, Color::White));
        self.squares[0][1] = Some(Piece::new(PieceType::Knight, Color::White));
        self.squares[0][2] = Some(Piece::new(PieceType::Bishop, Color::White));
        self.squares[0][3] = Some(Piece::new(PieceType::Queen, Color::White));
        self.squares[0][4] = Some(Piece::new(PieceType::King, Color::White));
        self.squares[0][5] = Some(Piece::new(PieceType::Bishop, Color::White));
        self.squares[0][6] = Some(Piece::new(PieceType::Knight, Color::White));
        self.squares[0][7] = Some(Piece::new(PieceType::Rook, Color::White));

        // Setup black pieces
        self.squares[7][0] = Some(Piece::new(PieceType::Rook, Color::Black));
        self.squares[7][1] = Some(Piece::new(PieceType::Knight, Color::Black));
        self.squares[7][2] = Some(Piece::new(PieceType::Bishop, Color::Black));
        self.squares[7][3] = Some(Piece::new(PieceType::Queen, Color::Black));
        self.squares[7][4] = Some(Piece::new(PieceType::King, Color::Black));
        self.squares[7][5] = Some(Piece::new(PieceType::Bishop, Color::Black));
        self.squares[7][6] = Some(Piece::new(PieceType::Knight, Color::Black));
        self.squares[7][7] = Some(Piece::new(PieceType::Rook, Color::Black));

        // Reset game state
        self.current_turn = Color::White;
        self.castling_rights = CastlingRights::new();
        self.en_passant_target = None;
        self.halfmove_clock = 0;
        self.fullmove_number = 1;
    }

    pub fn get_piece(&self, pos: Position) -> Option<Piece> {
        if !pos.is_valid() {
            return None;
        }
        self.squares[pos.row as usize][pos.col as usize]
    }

    pub fn set_piece(&mut self, pos: Position, piece: Option<Piece>) {
        if pos.is_valid() {
            self.squares[pos.row as usize][pos.col as usize] = piece;
        }
    }

    pub fn current_turn(&self) -> Color {
        self.current_turn
    }

    pub fn castling_rights(&self) -> &CastlingRights {
        &self.castling_rights
    }

    pub fn en_passant_target(&self) -> Option<Position> {
        self.en_passant_target
    }

    pub fn find_king(&self, color: Color) -> Option<Position> {
        for row in 0..8 {
            for col in 0..8 {
                let pos = Position::new(row, col);
                if let Some(piece) = self.get_piece(pos) {
                    if piece.color == color && piece.piece_type == PieceType::King {
                        return Some(pos);
                    }
                }
            }
        }
        None
    }

    pub fn make_move(&mut self, mv: Move) -> bool {
        let piece = match self.get_piece(mv.from) {
            Some(p) => p,
            None => return false,
        };

        if piece.color != self.current_turn {
            return false;
        }

        let captured_piece = self.get_piece(mv.to);

        // Handle en passant capture
        let is_en_passant = piece.piece_type == PieceType::Pawn
            && Some(mv.to) == self.en_passant_target
            && mv.from.col != mv.to.col
            && captured_piece.is_none();

        // Move the piece
        self.set_piece(mv.from, None);

        // Handle promotion
        let moving_piece = if let Some(promotion_type) = mv.promotion {
            Piece::new(promotion_type, piece.color)
        } else {
            piece
        };

        self.set_piece(mv.to, Some(moving_piece));

        // Handle en passant capture (remove the captured pawn)
        if is_en_passant {
            let captured_pawn_row = if piece.color == Color::White {
                mv.to.row - 1
            } else {
                mv.to.row + 1
            };
            self.set_piece(Position::new(captured_pawn_row, mv.to.col), None);
        }

        // Handle castling
        if piece.piece_type == PieceType::King && (mv.to.col - mv.from.col).abs() == 2 {
            let (rook_from_col, rook_to_col) = if mv.to.col > mv.from.col {
                // Kingside castling
                (7, 5)
            } else {
                // Queenside castling
                (0, 3)
            };
            let rook_row = mv.from.row;
            let rook = self.get_piece(Position::new(rook_row, rook_from_col));
            self.set_piece(Position::new(rook_row, rook_from_col), None);
            self.set_piece(Position::new(rook_row, rook_to_col), rook);
        }

        // Update en passant target
        self.en_passant_target = None;
        if piece.piece_type == PieceType::Pawn && (mv.to.row - mv.from.row).abs() == 2 {
            let en_passant_row = (mv.from.row + mv.to.row) / 2;
            self.en_passant_target = Some(Position::new(en_passant_row, mv.from.col));
        }

        // Update castling rights
        if piece.piece_type == PieceType::King {
            match piece.color {
                Color::White => {
                    self.castling_rights.white_kingside = false;
                    self.castling_rights.white_queenside = false;
                }
                Color::Black => {
                    self.castling_rights.black_kingside = false;
                    self.castling_rights.black_queenside = false;
                }
            }
        }

        if piece.piece_type == PieceType::Rook {
            match (piece.color, mv.from.col) {
                (Color::White, 0) => self.castling_rights.white_queenside = false,
                (Color::White, 7) => self.castling_rights.white_kingside = false,
                (Color::Black, 0) => self.castling_rights.black_queenside = false,
                (Color::Black, 7) => self.castling_rights.black_kingside = false,
                _ => {}
            }
        }

        // Update halfmove clock
        if piece.piece_type == PieceType::Pawn || captured_piece.is_some() {
            self.halfmove_clock = 0;
        } else {
            self.halfmove_clock += 1;
        }

        // Update move counters
        if self.current_turn == Color::Black {
            self.fullmove_number += 1;
        }

        // Switch turns
        self.current_turn = self.current_turn.opposite();

        true
    }

    /// Creates a copy of the board and makes a move on it
    pub fn make_move_copy(&self, mv: Move) -> Board {
        let mut new_board = self.clone();
        new_board.make_move(mv);
        new_board
    }

    /// Gets all pieces of a specific color
    pub fn get_pieces(&self, color: Color) -> Vec<(Position, Piece)> {
        let mut pieces = Vec::new();
        for row in 0..8 {
            for col in 0..8 {
                let pos = Position::new(row, col);
                if let Some(piece) = self.get_piece(pos) {
                    if piece.color == color {
                        pieces.push((pos, piece));
                    }
                }
            }
        }
        pieces
    }
}

impl Default for Board {
    fn default() -> Self {
        Self::new()
    }
}
