//! Agents for [Board]. Has a minimax agent and a random agent. Change between agents in the GUI or editing `Board.agent`
//!
//! # Minimax
//!
//! - Stored openings
//! - Alpha-beta pruning
//! - Sorted move ordering
//! - Transposition table
//!
//! # Random
//!
//! - Just picks a valid move by random
//!
//! # Control
//!
//! - Manually control the agent by clicking on the board

use macroquad::prelude::info;
use macroquad::rand::ChooseRandom;
use macroquad::time::get_time;
use rustc_hash::FxHashMap;

use crate::agent_opens::OPENINGS;
use crate::board::{Board, ChessColor};
use crate::board_eval::CHECKMATE_VALUE;
use crate::pieces::piece::PieceNames;
use crate::util::{choose_array, Loc};
use crate::{color_ternary, hashmap, ternary};

fn random_agent(board: &Board) -> Option<(Loc, Loc)> {
    let moves = board.moves(board.agent_color);
    moves.choose().copied()
}

/// What a transposition table score is worth, relative to the window it was searched with
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Bound {
    /// The true value of the position
    Exact,
    /// The search failed high, the true value is at least this
    Lower,
    /// The search failed low, the true value is at most this
    Upper,
}

/// `(depth, score, best move, bound)`
type TransEntry = (u8, i32, Option<(Loc, Loc)>, Bound);
type TransTable = FxHashMap<u64, TransEntry>;

fn should_store_transposition(stored_depth: Option<u8>, depth: u8) -> bool {
    stored_depth.map_or(true, |stored_depth| stored_depth < depth)
}

/// Head room for counting plies to mate, larger than any depth the search can reach
const MATE_BUFFER: i32 = u8::MAX as i32 + 1;

/// Score of a checkmate against `mated`, at the position where the mate actually happens
fn mate_score(mated: ChessColor) -> i32 {
    let magnitude = CHECKMATE_VALUE + MATE_BUFFER;
    color_ternary!(mated, -magnitude, magnitude)
}

/// Lift a child's score up one ply
///
/// - A mate score shrinks by one every ply it travels back up the tree. That makes the fastest
///   mate the biggest one, and, more importantly, keeps the score a property of the position it is
///   attached to rather than of the depth the search happened to start at. Scoring by the depth
///   left over instead made the same mate in one worth -20001 at depth 2 and -20004 at depth 5,
///   which is not a value the transposition table can hand back to a different search
fn from_child(score: i32) -> i32 {
    ternary!(is_mate_score(score), score - score.signum(), score)
}

/// True if `score` is a forced mate rather than a normal evaluation
fn is_mate_score(score: i32) -> bool {
    score.saturating_abs() > CHECKMATE_VALUE
}

/// How many plies of captures quiescence will follow before giving up and taking the static score
///
/// - Capture chains run out on their own in practice. This is insurance against a position where
///   they do not, and the one knob to turn if quiescence costs too much
const QUIESCENCE_MAX_PLY: u8 = 8;

/// How much a capture is allowed to fall short of `alpha` before quiescence stops looking at it
const DELTA_MARGIN: i32 = 200;

/// Whether a capture of a piece worth `captured` is hopeless: even taking it for free cannot pull
/// the stand-pat score into the window, so searching it cannot change the result
///
/// - Only ever consulted outside the endgame and never on a promotion, both of which move more
///   material than `captured` accounts for
fn delta_prunes(stand_pat: i32, captured: i32, alpha: i32, beta: i32, maximizing: bool) -> bool {
    ternary!(
        maximizing,
        stand_pat
            .saturating_add(captured)
            .saturating_add(DELTA_MARGIN)
            <= alpha,
        stand_pat
            .saturating_sub(captured)
            .saturating_sub(DELTA_MARGIN)
            >= beta
    )
}

/// Search the captures left over at a leaf, so a position is only ever scored once it is quiet
///
/// - Without this the search evaluates statically in the middle of an exchange, and scores the
///   half of it that happened to fit inside the depth. That is what made the engine play `Nxg7+`
///   and lose a knight to the recapture it never saw
/// - Same shape as [minimax]: `board.score` is from white's point of view, so white maximizes it,
///   and antimax flips who wants what. Returns a bare score, since a leaf has no move to report
#[allow(clippy::too_many_arguments)]
fn quiescence(
    board: &Board,
    mut alpha: i32,
    mut beta: i32,
    ply_left: u8,
    deadline: Option<f64>,
    antimax: bool,
    timed_out: &mut bool,
) -> i32 {
    // Draws are terminal wherever they turn up
    if board.is_over() {
        return board.score;
    }

    let maximizing = (board.turn == ChessColor::White) != antimax;
    let in_check = color_ternary!(board.turn, board.check_white, board.check_black);

    // Out of budget, so take the static score even though the position may not be quiet
    if ply_left == 0 {
        return board.score;
    }

    let moves;
    let mut best;
    let stand_pat;

    if in_check {
        // Standing pat is not on offer while the king is attacked, and searching only captures
        // would miss the quiet escapes, so this is a full width node
        moves = board.sorted_moves(board.turn);
        if moves.is_empty() {
            return mate_score(board.turn);
        }

        stand_pat = None;
        best = ternary!(maximizing, i32::MIN, i32::MAX);
    } else {
        // Stand pat: the side to move is never forced to capture, so the static score is a floor
        // for the maximizer and a ceiling for the minimizer
        best = board.score;

        if maximizing {
            if best >= beta {
                return best;
            }
            alpha = alpha.max(best);
        } else {
            if best <= alpha {
                return best;
            }
            beta = beta.min(best);
        }

        stand_pat = Some(best);
        moves = board.sorted_captures(board.turn);
    }

    for (from, to) in moves.iter() {
        if deadline.is_some_and(|deadline| get_time() > deadline) {
            *timed_out = true;
            return best;
        }

        // Delta pruning: a capture that cannot reach alpha even if the piece were a gift is not
        // worth searching. Off in the endgame, where a single pawn decides games and a two pawn
        // margin is no longer safe, and never on a promotion, whose swing the captured piece's
        // value does not account for
        if let Some(stand_pat) = stand_pat {
            if !board.endgame && !board.is_promotion(from, to) {
                if let Some(captured) = board.is_capture(from, to) {
                    let value = board.get(&captured).map_or(0, |piece| piece.value());
                    if delta_prunes(stand_pat, value, alpha, beta, maximizing) {
                        continue;
                    }
                }
            }
        }

        let mut test_board = board.clone();
        test_board.move_piece(from, to, false);

        let score = from_child(quiescence(
            &test_board,
            alpha,
            beta,
            ply_left - 1,
            deadline,
            antimax,
            timed_out,
        ));

        if *timed_out {
            return best;
        }

        if ternary!(maximizing, score > best, score < best) {
            best = score;
        }

        if maximizing {
            alpha = alpha.max(score);
        } else {
            beta = beta.min(score);
        }

        if alpha >= beta {
            break;
        }
    }

    best
}

/// Minimax agent with alpha-beta pruning and sorted move ordering
#[allow(clippy::type_complexity, clippy::too_many_arguments)]
fn minimax(
    board: &Board,
    depth: u8,
    mut alpha: i32,
    mut beta: i32,
    trans_table: &mut TransTable,
    deadline: Option<f64>,
    antimax: bool,
    timed_out: &mut bool,
) -> (i32, Option<(Loc, Loc)>) {
    // `board.score` is always from white's point of view, so white is the side that maximizes it.
    // Antimax plays for the opposite result, so it swaps who wants what
    let maximizing = (board.turn == ChessColor::White) != antimax;

    // Base case. `is_over` covers the draws, which are detected from the board alone (fifty move,
    // threefold, insufficient material). Checkmate and stalemate are found below, from the move
    // list, because `move_piece(.., false)` leaves `Board.moves_*` stale during the search
    if board.is_over() {
        return (board.score, None);
    }

    // At the horizon, hand off to quiescence rather than scoring a position that may be in the
    // middle of an exchange. Both base cases return before the transposition probe, so quiescence
    // scores never reach the table, which is keyed by a remaining depth they do not have
    if depth == 0 {
        let score = quiescence(
            board,
            alpha,
            beta,
            QUIESCENCE_MAX_PLY,
            deadline,
            antimax,
            timed_out,
        );

        return (score, None);
    }

    let alpha_orig = alpha;
    let beta_orig = beta;

    // Check if the current board state is already stored in the transposition table
    let should_store = match trans_table.get(&board.hash) {
        Some((stored_depth, stored_score, stored_best, bound)) => {
            // A stored score is only usable if the bound it was found under still applies to the
            // window being searched now, else a value that came out of a cutoff gets reused as if
            // it were exact
            if *stored_depth >= depth && stored_best.is_some() {
                let usable = match bound {
                    Bound::Exact => true,
                    Bound::Lower => *stored_score >= beta,
                    Bound::Upper => *stored_score <= alpha,
                };

                if usable {
                    return (*stored_score, *stored_best);
                }
            }

            should_store_transposition(Some(*stored_depth), depth)
        }
        None => should_store_transposition(None, depth),
    };

    // Get the sorted legal moves for the current turn
    let moves = board.sorted_moves(board.turn);

    // Nothing legal to play is checkmate if the king is attacked, and stalemate if it isn't
    if moves.is_empty() {
        let in_check = color_ternary!(board.turn, board.check_white, board.check_black);
        return (ternary!(in_check, mate_score(board.turn), 0), None);
    }

    let mut best_score = ternary!(maximizing, i32::MIN, i32::MAX);
    let mut best_move = None;

    // Iterate through the moves and apply minimax
    for (from, to) in moves.iter() {
        // Break if taking too long. The interrupted iteration is thrown away whole by
        // `minimax_agent`, so nothing half searched may be returned or stored
        if deadline.is_some_and(|deadline| get_time() > deadline) {
            *timed_out = true;
            return (best_score, best_move);
        }

        let mut test_board = board.clone();
        test_board.move_piece(from, to, false);

        let (score, _) = minimax(
            &test_board,
            depth - 1,
            alpha,
            beta,
            trans_table,
            deadline,
            antimax,
            timed_out,
        );
        let score = from_child(score);

        if *timed_out {
            return (best_score, best_move);
        }

        // Update the best score and best move
        if ternary!(maximizing, score > best_score, score < best_score) {
            best_score = score;
            best_move = Some((*from, *to));
        }

        // Update alpha and beta
        if maximizing {
            alpha = alpha.max(score);
        } else {
            beta = beta.min(score);
        }

        // Prune the search if alpha is greater than or equal to beta
        if alpha >= beta {
            break;
        }
    }

    // Store the data in the transposition table, tagged with how much the score can be trusted
    if should_store {
        let bound = if best_score <= alpha_orig {
            Bound::Upper
        } else if best_score >= beta_orig {
            Bound::Lower
        } else {
            Bound::Exact
        };

        trans_table.insert(board.hash, (depth, best_score, best_move, bound));
    }

    (best_score, best_move)
}

const MAX_TIME: f64 = 4.0;

/// Book move for the position, if there is one
///
/// - Only meaningful at the root. Probing the book inside the search made a book position deep in
///   the tree look like an outright win, and the parent would commit to the move leading there
fn book_move(board: &Board) -> Option<(Loc, Loc)> {
    // Very first move
    if board.full_moves() == 0 && board.agent_color == ChessColor::Black {
        macro_rules! responses {
            ($($key:expr => $value:expr,)+) => { responses!($($key => $value),+) };
            ($($key:expr => $value:expr),*) => {
                $(
                    if let Some(piece) = board.get(&Loc::from_notation($key.1)) {
                        if piece.name == $key.0 {
                            let m = choose_array(&$value);
                            info!("First move found!");
                            return Some((Loc::from_notation(m.0), Loc::from_notation(m.1)));
                        }
                    }
                )*
            };
        }

        responses! {
            // e4 -> e5, e6, c5
            (PieceNames::Pawn, "e4") => [("e7", "e5"), ("e7", "e6"), ("c7", "c5")],
            // d4 -> d5, c6, Nf6, Nc6
            (PieceNames::Pawn, "d4") => [("d7", "d5"), ("g8", "f6"), ("b8", "c6")],
            // c4 -> e5, Nf6
            (PieceNames::Pawn, "c4") => [("e7", "e5"), ("g8", "f6")],
            // Nf3 -> e5, Nf6
            (PieceNames::Knight, "f3") => [("e7", "e5"), ("g8", "f6")],
        };
    }

    // Openings
    if let Some(moves) = OPENINGS.get(&board.hash) {
        let (opening, name) = choose_array(moves);
        info!("Opening found! {}", name);
        return Some(*opening);
    }

    None
}

/// Wrapper for minimax, using iterative deepening
fn minimax_agent(board: &Board, antimax: bool) -> Option<(Loc, Loc)> {
    if board.is_over() {
        return None;
    }

    if !antimax {
        if let Some(opening) = book_move(board) {
            return Some(opening);
        }
    }

    let mut trans_table: TransTable = hashmap! {};
    let start_time = get_time();
    let mut last_time = start_time;

    let mut best_move = None;
    let mut depth: u8 = 0;
    loop {
        depth += 1;

        let mut timed_out = false;
        let (score, bm) = minimax(
            board,
            depth,
            i32::MIN,
            i32::MAX,
            &mut trans_table,
            Some(start_time + MAX_TIME),
            antimax,
            &mut timed_out,
        );

        let last_took = get_time() - last_time;
        last_time = get_time();
        let time_took = get_time() - start_time;

        // The interrupted depth only searched some of its moves, so it is discarded entirely and
        // the best move from the last depth that finished is kept
        if timed_out {
            info!(" - Timeout at depth {}", depth);
            break;
        }

        info!(
            "Depth: {} took {:.3}s (total: {:.3}s)",
            depth, last_took, time_took
        );

        if bm.is_some() {
            best_move = bm;
        }

        // A forced mate the agent is delivering can't be improved on by searching deeper. A mate
        // going the other way is worth searching on though, since a deeper search can still find a
        // longer defence
        if is_mate_score(score) {
            let winning = ternary!(board.turn == ChessColor::White, score > 0, score < 0);
            if winning != antimax {
                info!(" - Mate found at depth {}", depth);
                break;
            }
        }

        // Not enough time left to get through the next depth
        if time_took + last_took * 2.0 > MAX_TIME {
            info!(" - Last time timeout at depth {}", depth);
            break;
        }

        if depth == u8::MAX {
            break;
        }
    }

    best_move
}

/// List of agents for [Board] to use
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Agent {
    Minimax,
    Antimax,
    Control,
    Random,
}
impl Agent {
    pub(crate) fn get_move(&self, board: &Board) -> Option<(Loc, Loc)> {
        match self {
            Agent::Minimax => minimax_agent(board, false),
            Agent::Antimax => minimax_agent(board, true),
            Agent::Random => random_agent(board),
            Agent::Control => None,
        }
    }
}

pub(crate) const AGENTS: [(&str, Agent); 4] = [
    ("Random", Agent::Random),
    ("Control", Agent::Control),
    ("Antimax", Agent::Antimax),
    ("Minimax", Agent::Minimax),
];

#[cfg(test)]
mod tests {
    use super::*;
    use crate::board::BoardState;

    /// Search a position with no time limit, so the tests don't need a macroquad clock
    fn search(fen: &str, depth: u8) -> (i32, Option<(Loc, Loc)>) {
        let board = Board::from_fen(fen);
        let mut trans_table: TransTable = hashmap! {};
        let mut timed_out = false;

        let result = minimax(
            &board,
            depth,
            i32::MIN,
            i32::MAX,
            &mut trans_table,
            None,
            false,
            &mut timed_out,
        );

        assert!(!timed_out, "search should not time out without a deadline");
        result
    }

    #[test]
    fn faster_mates_score_higher_than_slower_ones() {
        // Two plies further from the mate is two points less of a mate score
        let near = mate_score(ChessColor::White);
        let far = from_child(from_child(near));

        assert!(
            far > near,
            "a slower mate against white has to score higher"
        );
        assert!(is_mate_score(near) && is_mate_score(far));

        assert!(!is_mate_score(0));
        assert!(!is_mate_score(-900));

        // Ordinary scores travel up untouched
        assert_eq!(from_child(42), 42);
    }

    #[test]
    fn search_finds_mate_in_one() {
        // Black to move, Rb1 is mate. Needs depth 2: the mate is only visible once white has been
        // given a turn and found to have no legal reply
        let (score, best) = search("1r5k/8/8/8/8/8/r7/7K b - - 0 30", 2);

        assert!(is_mate_score(score), "expected a mate score, got {}", score);
        assert!(score < 0, "white is the side being mated");
        assert_eq!(
            best,
            Some((Loc::from_notation("b8"), Loc::from_notation("b1")))
        );
    }

    #[test]
    fn search_avoids_stalemating_when_it_can_mate() {
        // Regression for the terminal detection: with stale move lists the search saw neither the
        // mate nor the stalemate, and scored both as an ordinary position
        let board = Board::from_fen("1r5k/8/8/8/8/8/r7/7K b - - 0 30");
        let (_, best) = search("1r5k/8/8/8/8/8/r7/7K b - - 0 30", 2);

        let (from, to) = best.unwrap();
        let mut played = board;
        played.move_piece(&from, &to, true);

        assert_eq!(played.state, BoardState::Checkmate(ChessColor::White));
    }

    #[test]
    fn search_prefers_a_won_position_over_a_draw() {
        // White is up a rook with an easy win, so the draw the repetition offers is not a prize.
        // This used to fail because a draw scored -100, which minimizing black actively wanted
        let (score, _) = search("7k/8/8/8/8/8/R7/6RK w - - 0 30", 2);

        assert!(score > 0, "white is winning, got {}", score);
    }

    #[test]
    fn transposition_entries_store_when_missing() {
        assert!(should_store_transposition(None, 3));
    }

    #[test]
    fn transposition_entries_store_when_new_depth_is_greater() {
        assert!(should_store_transposition(Some(2), 3));
    }

    #[test]
    fn transposition_entries_do_not_store_when_existing_depth_is_deeper() {
        assert!(!should_store_transposition(Some(3), 2));
        assert!(!should_store_transposition(Some(3), 3));
    }

    /// Plain minimax, no pruning and no transposition table. Alpha-beta and the table are only
    /// ever allowed to make the search *faster*, so they have to agree with this exactly
    fn reference(board: &Board, depth: u8) -> i32 {
        if board.is_over() {
            return board.score;
        }

        // The leaves have to be scored the same way the real search scores them, else this is
        // measuring quiescence rather than the pruning it is meant to be checking. The window is
        // wide open here, so quiescence does no alpha-beta cutting of its own
        if depth == 0 {
            let mut timed_out = false;
            return quiescence(
                board,
                i32::MIN,
                i32::MAX,
                QUIESCENCE_MAX_PLY,
                None,
                false,
                &mut timed_out,
            );
        }

        let moves = board.sorted_moves(board.turn);
        if moves.is_empty() {
            let in_check = color_ternary!(board.turn, board.check_white, board.check_black);
            return ternary!(in_check, mate_score(board.turn), 0);
        }

        let maximizing = board.turn == ChessColor::White;
        let mut best = ternary!(maximizing, i32::MIN, i32::MAX);

        for (from, to) in moves.iter() {
            let mut next = board.clone();
            next.move_piece(from, to, false);

            let score = from_child(reference(&next, depth - 1));
            if ternary!(maximizing, score > best, score < best) {
                best = score;
            }
        }

        best
    }

    #[test]
    fn pruning_and_the_transposition_table_do_not_change_the_score() {
        // Entries used to be stored without recording whether the score came out of a cutoff, so a
        // bound got reused as if it were the exact value of the position
        //
        // The depths are modest because `reference` prunes nothing and still runs a full
        // quiescence at every one of its leaves, which grows a lot faster than the real search
        let positions = [
            // Opening, lots of transpositions between move orders
            (crate::conf::DEFAULT_FEN, 3),
            // Middlegame with hanging pieces on both sides, so quiescence has real work to do
            (
                "r3kb1r/ppp1npp1/1qn4p/3p3N/5Pb1/P2B1N2/1PPPQ1PP/R1B1K2R w KQkq - 1 14",
                2,
            ),
            // Knights and kings only: almost every move is reversible, so the same positions
            // turn up over and over at different remaining depths
            ("4k3/8/1n4n1/8/8/1N4N1/8/4K3 w - - 0 30", 3),
            // Endgame, deep enough to find the mate
            ("1r5k/8/8/8/8/8/r7/7K b - - 0 30", 4),
        ];

        for (fen, depth) in positions {
            let expected = reference(&Board::from_fen(fen), depth);
            let (score, _) = search(fen, depth);

            assert_eq!(score, expected, "at depth {} in {}", depth, fen);
        }
    }

    #[test]
    fn a_mate_scores_the_same_at_every_search_depth() {
        // The transposition table hands a stored score to searches of other depths, so a mate has
        // to be worth the same however deep the search that found it was. Scoring by the depth
        // left over made this same mate in one worth -20001 at depth 2 and -20004 at depth 5
        let fen = "1r5k/8/8/8/8/8/r7/7K b - - 0 30";
        let expected = search(fen, 2).0;

        assert!(is_mate_score(expected));
        for depth in [3, 4, 5] {
            assert_eq!(search(fen, depth).0, expected, "at depth {}", depth);
        }
    }

    /// Run quiescence on its own, with the window wide open and no clock
    fn quiesce(board: &Board) -> i32 {
        let mut timed_out = false;
        let score = quiescence(
            board,
            i32::MIN,
            i32::MAX,
            QUIESCENCE_MAX_PLY,
            None,
            false,
            &mut timed_out,
        );

        assert!(
            !timed_out,
            "quiescence should not time out without a deadline"
        );
        score
    }

    /// The position that motivated quiescence, from a real self play game
    const HORIZON_FEN: &str =
        "r3kb1r/ppp1npp1/1qn4p/3p3N/5Pb1/P2B1N2/1PPPQ1PP/R1B1K2R w KQkq - 1 14";

    #[test]
    fn does_not_hang_a_knight_over_the_horizon() {
        // Nxg7 wins a pawn and then loses the knight to Bxg7. Without quiescence the search
        // stopped in between and scored the position at +62, so it played it
        let (_, best) = search(HORIZON_FEN, 3);

        assert_ne!(
            best,
            Some((Loc::from_notation("h5"), Loc::from_notation("g7"))),
            "Nxg7 hangs a knight to the recapture"
        );
    }

    #[test]
    fn scores_stay_steady_from_one_depth_to_the_next() {
        // Evaluating in the middle of an exchange made this position swing between -380 and +300
        // depending on who got the last capture inside the depth. A quiet leaf does not do that
        let scores = (2..=5)
            .map(|d| search(HORIZON_FEN, d).0)
            .collect::<Vec<_>>();

        let swing = scores.iter().max().unwrap() - scores.iter().min().unwrap();
        assert!(
            swing < 100,
            "scores still swinging by {}: {:?}",
            swing,
            scores
        );
    }

    #[test]
    fn quiescence_does_not_stand_pat_while_in_check() {
        // White is in check from the rook and every escape is a quiet king move. Standing pat, or
        // looking only at captures, would score the position without ever escaping
        let board = Board::from_fen("4k3/8/8/8/8/8/8/r3K3 w - - 0 30");
        assert!(board.check_white);

        let score = quiesce(&board);

        assert_ne!(score, board.score, "stood pat while in check");
        assert!(!is_mate_score(score), "this is check, not mate");
    }

    #[test]
    fn quiescence_reports_checkmate() {
        // Reached through `move_piece(.., false)`, the search path, so the board itself only knows
        // it is check. Quiescence has to work the mate out from the empty move list
        let mut board = Board::from_fen("1r5k/8/8/8/8/8/r7/7K b - - 0 30");
        board.move_piece(&Loc::from_notation("b8"), &Loc::from_notation("b1"), false);
        assert!(!board.is_over(), "the search path never sets Checkmate");

        let score = quiesce(&board);

        assert!(is_mate_score(score), "expected a mate score, got {}", score);
        assert!(score < 0, "white is the side being mated");
    }

    #[test]
    fn captures_include_promotions() {
        // A promotion is not a capture but swings as much material as one, so quiescence has to
        // see it
        let board = Board::from_fen("4k3/P7/8/8/8/8/8/4K3 w - - 0 30");

        assert!(board
            .sorted_captures(ChessColor::White)
            .contains(&(Loc::from_notation("a7"), Loc::from_notation("a8"))));
    }

    #[test]
    fn delta_pruning_only_skips_hopeless_captures() {
        // Maximizing: 100 behind, winning a queen would clear alpha, winning a pawn would not
        assert!(!delta_prunes(-100, 900, 0, i32::MAX, true));
        assert!(delta_prunes(-2000, 100, 0, i32::MAX, true));

        // Minimizing is the mirror image
        assert!(!delta_prunes(100, 900, i32::MIN, 0, false));
        assert!(delta_prunes(2000, 100, i32::MIN, 0, false));

        // The margin is the boundary: exactly short of alpha by it is pruned, a point better is not
        assert!(delta_prunes(-DELTA_MARGIN, 0, 0, i32::MAX, true));
        assert!(!delta_prunes(-DELTA_MARGIN + 1, 0, 0, i32::MAX, true));

        // A wide open window never prunes, and nothing overflows at the extremes
        assert!(!delta_prunes(0, 100, i32::MIN, i32::MAX, true));
        assert!(!delta_prunes(0, 100, i32::MIN, i32::MAX, false));
    }

    #[test]
    fn delta_pruning_is_off_in_the_endgame() {
        // Thin enough that a single pawn decides it, which is exactly where a two pawn margin
        // stops being safe. exd5 has to be found
        let board = Board::from_fen("8/8/8/3p4/4P3/8/8/K6k w - - 0 30");
        assert!(board.endgame, "the delta pruning guard keys off this flag");

        let (_, best) = search("8/8/8/3p4/4P3/8/8/K6k w - - 0 30", 4);

        assert_eq!(
            best,
            Some((Loc::from_notation("e4"), Loc::from_notation("d5")))
        );
    }
}
