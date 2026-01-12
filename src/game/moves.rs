use super::board::Board;
use super::piece::{Color, Move, PieceType, Position};

/// Generates all pseudo-legal moves for a piece at the given position
/// Pseudo-legal means the moves follow piece movement rules but may leave the king in check
pub fn generate_pseudo_legal_moves(board: &Board, from: Position) -> Vec<Move> {
    let piece = match board.get_piece(from) {
        Some(p) => p,
        None => return Vec::new(),
    };

    match piece.piece_type {
        PieceType::Pawn => generate_pawn_moves(board, from, piece.color),
        PieceType::Knight => generate_knight_moves(board, from, piece.color),
        PieceType::Bishop => generate_bishop_moves(board, from, piece.color),
        PieceType::Rook => generate_rook_moves(board, from, piece.color),
        PieceType::Queen => generate_queen_moves(board, from, piece.color),
        PieceType::King => generate_king_moves(board, from, piece.color),
    }
}

fn generate_pawn_moves(board: &Board, from: Position, color: Color) -> Vec<Move> {
    let mut moves = Vec::new();
    let direction = if color == Color::White { 1 } else { -1 };
    let start_row = if color == Color::White { 1 } else { 6 };
    let promotion_row = if color == Color::White { 7 } else { 0 };

    // Forward move
    let one_forward = Position::new(from.row + direction, from.col);
    if one_forward.is_valid() && board.get_piece(one_forward).is_none() {
        if one_forward.row == promotion_row {
            // Add all promotion options
            for promotion_type in [
                PieceType::Queen,
                PieceType::Rook,
                PieceType::Bishop,
                PieceType::Knight,
            ] {
                moves.push(Move::with_promotion(from, one_forward, promotion_type));
            }
        } else {
            moves.push(Move::new(from, one_forward));
        }

        // Double forward move from starting position
        if from.row == start_row {
            let two_forward = Position::new(from.row + 2 * direction, from.col);
            if two_forward.is_valid() && board.get_piece(two_forward).is_none() {
                moves.push(Move::new(from, two_forward));
            }
        }
    }

    // Captures
    for col_offset in [-1, 1] {
        let capture_pos = Position::new(from.row + direction, from.col + col_offset);
        if !capture_pos.is_valid() {
            continue;
        }

        let can_capture = if let Some(target) = board.get_piece(capture_pos) {
            target.color != color
        } else {
            // En passant
            Some(capture_pos) == board.en_passant_target()
        };

        if can_capture {
            if capture_pos.row == promotion_row {
                // Capture with promotion
                for promotion_type in [
                    PieceType::Queen,
                    PieceType::Rook,
                    PieceType::Bishop,
                    PieceType::Knight,
                ] {
                    moves.push(Move::with_promotion(from, capture_pos, promotion_type));
                }
            } else {
                moves.push(Move::new(from, capture_pos));
            }
        }
    }

    moves
}

fn generate_knight_moves(board: &Board, from: Position, color: Color) -> Vec<Move> {
    let mut moves = Vec::new();
    let knight_offsets = [
        (-2, -1),
        (-2, 1),
        (-1, -2),
        (-1, 2),
        (1, -2),
        (1, 2),
        (2, -1),
        (2, 1),
    ];

    for (row_offset, col_offset) in knight_offsets {
        let to = Position::new(from.row + row_offset, from.col + col_offset);
        if !to.is_valid() {
            continue;
        }

        if let Some(target) = board.get_piece(to) {
            if target.color != color {
                moves.push(Move::new(from, to));
            }
        } else {
            moves.push(Move::new(from, to));
        }
    }

    moves
}

fn generate_bishop_moves(board: &Board, from: Position, color: Color) -> Vec<Move> {
    let directions = [(-1, -1), (-1, 1), (1, -1), (1, 1)];
    generate_sliding_moves(board, from, color, &directions)
}

fn generate_rook_moves(board: &Board, from: Position, color: Color) -> Vec<Move> {
    let directions = [(-1, 0), (1, 0), (0, -1), (0, 1)];
    generate_sliding_moves(board, from, color, &directions)
}

fn generate_queen_moves(board: &Board, from: Position, color: Color) -> Vec<Move> {
    let directions = [
        (-1, -1),
        (-1, 0),
        (-1, 1),
        (0, -1),
        (0, 1),
        (1, -1),
        (1, 0),
        (1, 1),
    ];
    generate_sliding_moves(board, from, color, &directions)
}

fn generate_sliding_moves(
    board: &Board,
    from: Position,
    color: Color,
    directions: &[(i8, i8)],
) -> Vec<Move> {
    let mut moves = Vec::new();

    for &(row_dir, col_dir) in directions {
        let mut current_row = from.row + row_dir;
        let mut current_col = from.col + col_dir;

        while current_row >= 0 && current_row < 8 && current_col >= 0 && current_col < 8 {
            let to = Position::new(current_row, current_col);

            if let Some(target) = board.get_piece(to) {
                if target.color != color {
                    moves.push(Move::new(from, to));
                }
                break; // Can't move past a piece
            } else {
                moves.push(Move::new(from, to));
            }

            current_row += row_dir;
            current_col += col_dir;
        }
    }

    moves
}

fn generate_king_moves(board: &Board, from: Position, color: Color) -> Vec<Move> {
    let mut moves = Vec::new();
    let king_offsets = [
        (-1, -1),
        (-1, 0),
        (-1, 1),
        (0, -1),
        (0, 1),
        (1, -1),
        (1, 0),
        (1, 1),
    ];

    // Normal king moves
    for (row_offset, col_offset) in king_offsets {
        let to = Position::new(from.row + row_offset, from.col + col_offset);
        if !to.is_valid() {
            continue;
        }

        if let Some(target) = board.get_piece(to) {
            if target.color != color {
                moves.push(Move::new(from, to));
            }
        } else {
            moves.push(Move::new(from, to));
        }
    }

    // Castling moves
    let rights = board.castling_rights();
    let back_row = if color == Color::White { 0 } else { 7 };

    if from.row == back_row && from.col == 4 {
        // Kingside castling
        let can_castle_kingside = match color {
            Color::White => rights.white_kingside,
            Color::Black => rights.black_kingside,
        };

        if can_castle_kingside {
            let f_square = Position::new(back_row, 5);
            let g_square = Position::new(back_row, 6);

            if board.get_piece(f_square).is_none() && board.get_piece(g_square).is_none() {
                moves.push(Move::new(from, g_square));
            }
        }

        // Queenside castling
        let can_castle_queenside = match color {
            Color::White => rights.white_queenside,
            Color::Black => rights.black_queenside,
        };

        if can_castle_queenside {
            let d_square = Position::new(back_row, 3);
            let c_square = Position::new(back_row, 2);
            let b_square = Position::new(back_row, 1);

            if board.get_piece(d_square).is_none()
                && board.get_piece(c_square).is_none()
                && board.get_piece(b_square).is_none()
            {
                moves.push(Move::new(from, c_square));
            }
        }
    }

    moves
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pawn_initial_moves() {
        let board = Board::new();
        let moves = generate_pseudo_legal_moves(&board, Position::new(1, 4));
        assert_eq!(moves.len(), 2); // Can move 1 or 2 squares forward
    }

    #[test]
    fn test_knight_moves() {
        let board = Board::new();
        let moves = generate_pseudo_legal_moves(&board, Position::new(0, 1));
        assert_eq!(moves.len(), 2); // Knight on b1 can move to a3 or c3
    }
}
