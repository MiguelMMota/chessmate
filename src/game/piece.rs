use serde::{Deserialize, Serialize};

/// Represents an action that occurred on the board
/// Used to communicate what happened to clients for animation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GameAction {
    /// Simple move without capture
    Move {
        piece_id: u8,
        from: Position,
        to: Position,
    },
    /// Move that captures an opponent piece
    Capture {
        attacker_id: u8,
        victim_id: u8,
        from: Position,
        to: Position,
    },
    /// Castling move
    Castle {
        king_id: u8,
        rook_id: u8,
        king_from: Position,
        king_to: Position,
        rook_from: Position,
        rook_to: Position,
        side: CastleSide,
    },
    /// En passant capture
    EnPassant {
        pawn_id: u8,
        captured_pawn_id: u8,
        from: Position,
        to: Position,
        captured_pawn_pos: Position,
    },
    /// Pawn promotion (may include capture)
    /// The pawn is destroyed and a new piece is created
    Promotion {
        old_pawn_id: u8,
        new_piece_id: u8,
        from: Position,
        to: Position,
        new_piece_type: PieceType,
        captured_piece_id: Option<u8>, // Some if promotion captures
    },
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum CastleSide {
    Kingside,
    Queenside,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PieceType {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Color {
    White,
    Black,
}

impl Color {
    pub fn opposite(&self) -> Color {
        match self {
            Color::White => Color::Black,
            Color::Black => Color::White,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Piece {
    pub piece_type: PieceType,
    pub color: Color,
    pub id: u8, // Unique ID: 0-15 for White, 16-31 for Black
}

impl Piece {
    pub fn new(piece_type: PieceType, color: Color, id: u8) -> Self {
        Self {
            piece_type,
            color,
            id,
        }
    }

    /// Returns the Unicode chess symbol for this piece (filled style for both colors)
    pub fn to_symbol(&self) -> char {
        match self.piece_type {
            PieceType::King => '♚',
            PieceType::Queen => '♛',
            PieceType::Rook => '♜',
            PieceType::Bishop => '♝',
            PieceType::Knight => '♞',
            PieceType::Pawn => '♟',
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Position {
    pub row: i8,
    pub col: i8,
}

impl Position {
    pub fn new(row: i8, col: i8) -> Self {
        Self { row, col }
    }

    pub fn is_valid(&self) -> bool {
        self.row >= 0 && self.row < 8 && self.col >= 0 && self.col < 8
    }

    pub fn from_algebraic(notation: &str) -> Option<Self> {
        if notation.len() != 2 {
            return None;
        }
        let chars: Vec<char> = notation.chars().collect();
        let col = (chars[0] as i8) - ('a' as i8);
        let row = (chars[1] as i8) - ('1' as i8);

        let pos = Position::new(row, col);
        if pos.is_valid() {
            Some(pos)
        } else {
            None
        }
    }

    pub fn to_algebraic(&self) -> String {
        let col_char = (b'a' + self.col as u8) as char;
        let row_char = (b'1' + self.row as u8) as char;
        format!("{}{}", col_char, row_char)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Move {
    pub from: Position,
    pub to: Position,
    pub promotion: Option<PieceType>,
}

impl Move {
    pub fn new(from: Position, to: Position) -> Self {
        Self {
            from,
            to,
            promotion: None,
        }
    }

    pub fn with_promotion(from: Position, to: Position, promotion: PieceType) -> Self {
        Self {
            from,
            to,
            promotion: Some(promotion),
        }
    }
}
