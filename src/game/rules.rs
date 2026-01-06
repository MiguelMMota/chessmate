use super::board::{Board, GameStatus};
use super::moves::generate_pseudo_legal_moves;
use super::piece::{Color, PieceType, Position, Move};
use std::collections::HashMap;

/// Check if a square is under attack by the given color
pub fn is_square_attacked(board: &Board, square: Position, by_color: Color) -> bool {
    // Check all pieces of the attacking color
    for row in 0..8 {
        for col in 0..8 {
            let from = Position::new(row, col);
            if let Some(piece) = board.get_piece(from) {
                if piece.color == by_color {
                    // Generate pseudo-legal moves for this piece
                    let moves = generate_pseudo_legal_moves(board, from);
                    for mv in moves {
                        if mv.to == square {
                            // Special handling for pawns (they attack diagonally but move straight)
                            if piece.piece_type == PieceType::Pawn {
                                // Pawn attacks diagonally
                                if mv.from.col != mv.to.col {
                                    return true;
                                }
                            } else {
                                return true;
                            }
                        }
                    }
                }
            }
        }
    }
    false
}

/// Check if the king of the given color is in check
pub fn is_in_check(board: &Board, color: Color) -> bool {
    if let Some(king_pos) = board.find_king(color) {
        is_square_attacked(board, king_pos, color.opposite())
    } else {
        false
    }
}

/// Check if a move is legal (doesn't leave the king in check)
pub fn is_move_legal(board: &Board, mv: Move) -> bool {
    let piece = match board.get_piece(mv.from) {
        Some(p) => p,
        None => return false,
    };

    // Make the move on a copy of the board
    let new_board = board.make_move_copy(mv);

    // Check if the moving player's king is in check after the move
    // Note: make_move_copy switches the turn, so we need to check the opposite color
    !is_in_check(&new_board, piece.color)
}

/// Special validation for castling moves
pub fn is_castling_legal(board: &Board, mv: Move) -> bool {
    let piece = match board.get_piece(mv.from) {
        Some(p) => p,
        None => return false,
    };

    if piece.piece_type != PieceType::King {
        return true; // Not a castling move
    }

    let col_diff = mv.to.col - mv.from.col;
    if col_diff.abs() != 2 {
        return true; // Not a castling move
    }

    // Can't castle out of check
    if is_in_check(board, piece.color) {
        return false;
    }

    // Can't castle through check
    let intermediate_col = (mv.from.col + mv.to.col) / 2;
    let intermediate_pos = Position::new(mv.from.row, intermediate_col);

    // Create a temporary board with king moved to intermediate square
    let mut temp_board = board.clone();
    temp_board.set_piece(mv.from, None);
    temp_board.set_piece(intermediate_pos, Some(piece));

    if is_in_check(&temp_board, piece.color) {
        return false;
    }

    // Can't castle into check (this is checked by is_move_legal)
    true
}

/// Generate all legal moves for a piece
pub fn generate_legal_moves(board: &Board, from: Position) -> Vec<Move> {
    let pseudo_legal_moves = generate_pseudo_legal_moves(board, from);

    pseudo_legal_moves
        .into_iter()
        .filter(|&mv| {
            // First check castling-specific rules
            if !is_castling_legal(board, mv) {
                return false;
            }
            // Then check if the move leaves the king in check
            is_move_legal(board, mv)
        })
        .collect()
}

/// Generate all legal moves for the current player
pub fn generate_all_legal_moves(board: &Board) -> Vec<Move> {
    let current_color = board.current_turn();
    let pieces = board.get_pieces(current_color);

    let mut all_moves = Vec::new();
    for (pos, _) in pieces {
        let moves = generate_legal_moves(board, pos);
        all_moves.extend(moves);
    }

    all_moves
}

/// Check for insufficient material draw conditions
pub fn has_insufficient_material(board: &Board) -> bool {
    let mut piece_counts: HashMap<(Color, PieceType), u32> = HashMap::new();
    let mut bishop_colors: HashMap<Color, Vec<bool>> = HashMap::new(); // Track bishop square colors

    for row in 0..8 {
        for col in 0..8 {
            let pos = Position::new(row, col);
            if let Some(piece) = board.get_piece(pos) {
                *piece_counts.entry((piece.color, piece.piece_type)).or_insert(0) += 1;

                // Track square color for bishops (white squares have even row+col sum)
                if piece.piece_type == PieceType::Bishop {
                    let is_white_square = (row + col) % 2 == 0;
                    bishop_colors.entry(piece.color).or_insert_with(Vec::new).push(is_white_square);
                }
            }
        }
    }

    // Get total piece count (excluding kings)
    let total_pieces: u32 = piece_counts
        .iter()
        .filter(|((_, piece_type), _)| *piece_type != PieceType::King)
        .map(|(_, count)| count)
        .sum();

    // King vs King
    if total_pieces == 0 {
        return true;
    }

    // King + Bishop vs King or King + Knight vs King
    if total_pieces == 1 {
        return piece_counts.get(&(Color::White, PieceType::Bishop)).is_some()
            || piece_counts.get(&(Color::Black, PieceType::Bishop)).is_some()
            || piece_counts.get(&(Color::White, PieceType::Knight)).is_some()
            || piece_counts.get(&(Color::Black, PieceType::Knight)).is_some();
    }

    // King + Bishop vs King + Bishop (same color squares)
    if total_pieces == 2 {
        let white_bishops = piece_counts.get(&(Color::White, PieceType::Bishop)).unwrap_or(&0);
        let black_bishops = piece_counts.get(&(Color::Black, PieceType::Bishop)).unwrap_or(&0);

        if *white_bishops == 1 && *black_bishops == 1 {
            // Check if bishops are on same color squares
            if let (Some(white_squares), Some(black_squares)) =
                (bishop_colors.get(&Color::White), bishop_colors.get(&Color::Black)) {
                if white_squares[0] == black_squares[0] {
                    return true;
                }
            }
        }
    }

    false
}

/// Determine the current game status
pub fn get_game_status(board: &Board) -> GameStatus {
    let current_color = board.current_turn();
    let legal_moves = generate_all_legal_moves(board);
    let in_check = is_in_check(board, current_color);

    // Checkmate: in check with no legal moves
    if in_check && legal_moves.is_empty() {
        return GameStatus::Checkmate(current_color.opposite());
    }

    // Stalemate: not in check but no legal moves
    if !in_check && legal_moves.is_empty() {
        return GameStatus::Stalemate;
    }

    // Check for insufficient material
    if has_insufficient_material(board) {
        return GameStatus::DrawInsufficientMaterial;
    }

    // Check if in check (but not checkmate)
    if in_check {
        return GameStatus::Check;
    }

    GameStatus::Ongoing
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initial_position_not_in_check() {
        let board = Board::new();
        assert!(!is_in_check(&board, Color::White));
        assert!(!is_in_check(&board, Color::Black));
    }

    #[test]
    fn test_insufficient_material_king_vs_king() {
        let mut board = Board::new();
        // Clear all pieces except kings
        for row in 0..8 {
            for col in 0..8 {
                let pos = Position::new(row, col);
                if let Some(piece) = board.get_piece(pos) {
                    if piece.piece_type != PieceType::King {
                        board.set_piece(pos, None);
                    }
                }
            }
        }
        assert!(has_insufficient_material(&board));
    }
}
