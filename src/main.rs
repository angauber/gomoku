mod goban;

use crate::goban::{Axis, Cell, Goban, Player, WIN_MINIMUM_LINE_SIZE};

fn main() {
    let mut goban: Goban = Goban::new();

    for i in 0..WIN_MINIMUM_LINE_SIZE {
        goban.set(i,  18 - i, Cell::Computer);
    }

    println!("{:?}", goban);
}
