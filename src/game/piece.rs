#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PieceType {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Piece {
    pub piece_type: PieceType,
    pub color: Color,
}

impl Piece {
    pub fn new(piece_type: PieceType, color: Color) -> Self {
        Self { piece_type, color }
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
