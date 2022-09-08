use std::collections::HashMap;

use crate::evaluator::Eval;
use crate::goban::Player;
use crate::zobrist_hashing::ZobristHash;

pub type TranspositionTable = HashMap<(ZobristHash, Player), Eval>;