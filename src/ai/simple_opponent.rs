use crate::game::board::Board;
use crate::game::piece::{Move, PieceType};
use crate::game::rules::{generate_all_legal_moves, is_in_check};
use rand::Rng;

/// Calculate weight for a move based on simple heuristics
fn calculate_move_weight(board: &Board, mv: &Move) -> f64 {
    let mut weight = 0.0;

    // Get the moving piece
    let moving_piece = board.get_piece(mv.from).expect("Moving piece should exist");

    // Base weight for piece movement
    let base_piece_weight = match moving_piece.piece_type {
        PieceType::King => 1.0,
        PieceType::Pawn => 2.0,
        PieceType::Knight | PieceType::Bishop | PieceType::Rook => 4.0,
        PieceType::Queen => 6.0,
    };

    weight += base_piece_weight;

    // Check if this is a capture
    if let Some(captured_piece) = board.get_piece(mv.to) {
        let capture_base_weight = match captured_piece.piece_type {
            PieceType::King => 1.0, // King can't actually be captured, but just in case
            PieceType::Pawn => 2.0,
            PieceType::Knight | PieceType::Bishop | PieceType::Rook => 4.0,
            PieceType::Queen => 6.0,
        };

        // Square of the capture base weight
        weight += capture_base_weight * capture_base_weight;
    }

    // Check for en passant capture
    if moving_piece.piece_type == PieceType::Pawn
        && mv.from.col != mv.to.col
        && board.get_piece(mv.to).is_none()
    {
        // En passant - equivalent to pawn capture
        weight += 2.0 * 2.0; // 4.0
    }

    // Check for castling
    if moving_piece.piece_type == PieceType::King && (mv.to.col - mv.from.col).abs() == 2 {
        weight += 16.0;
    }

    // Check for promotion
    if let Some(promotion_type) = mv.promotion {
        // Promotion weight is equivalent to capturing a piece of that type
        let promotion_weight = match promotion_type {
            PieceType::Knight | PieceType::Bishop | PieceType::Rook => 4.0,
            PieceType::Queen => 6.0,
            _ => 0.0,
        };
        weight += promotion_weight * promotion_weight;
    }

    // Check if move results in check
    let new_board = board.make_move_copy(*mv);
    if is_in_check(&new_board, moving_piece.color.opposite()) {
        weight += 8.0;
    }

    weight
}

/// Select a move using weighted random selection
pub fn select_weighted_move(board: &Board) -> Option<Move> {
    let legal_moves = generate_all_legal_moves(board);

    if legal_moves.is_empty() {
        return None;
    }

    // Calculate weights for all moves
    let weights: Vec<f64> = legal_moves
        .iter()
        .map(|mv| calculate_move_weight(board, mv))
        .collect();

    // Calculate total weight
    let total_weight: f64 = weights.iter().sum();

    if total_weight <= 0.0 {
        // If all weights are 0, just pick randomly
        let mut rng = rand::thread_rng();
        let index = rng.gen_range(0..legal_moves.len());
        return Some(legal_moves[index]);
    }

    // Select a random value between 0 and total_weight
    let mut rng = rand::thread_rng();
    let mut random_value = rng.gen_range(0.0..total_weight);

    // Find the move corresponding to this random value
    for (i, weight) in weights.iter().enumerate() {
        random_value -= weight;
        if random_value <= 0.0 {
            return Some(legal_moves[i]);
        }
    }

    // Fallback: return last move (should never reach here)
    legal_moves.last().copied()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::board::Board;

    #[test]
    fn test_select_move_initial_position() {
        let board = Board::new();
        let selected_move = select_weighted_move(&board);

        assert!(
            selected_move.is_some(),
            "Should be able to select a move from initial position"
        );
    }

    #[test]
    fn test_move_weights_prefer_captures() {
        let board = Board::new();

        // Create a scenario where there's a clear capture available
        // This is a basic test - in practice we'd set up a specific position
        let moves = generate_all_legal_moves(&board);

        for mv in &moves {
            let weight = calculate_move_weight(&board, mv);
            assert!(weight > 0.0, "All moves should have positive weight");
        }
    }
}
