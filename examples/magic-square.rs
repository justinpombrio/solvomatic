//! Find all 4x4 associative magic squares

use solvomatic::constraints::{Permutation, Pred, Sum};
use solvomatic::{Solvomatic, State};
use std::fmt;

/// An NxN magic square.
#[derive(Debug)]
struct MagicSquare<const N: usize>([[Option<u8>; N]; N]);

impl<const N: usize> Default for MagicSquare<N> {
    fn default() -> MagicSquare<N> {
        MagicSquare([[None; N]; N])
    }
}

impl<const N: usize> State for MagicSquare<N> {
    type Var = (i8, i8);
    type Value = u8;
    type MetaData = ();

    fn set(&mut self, var: (i8, i8), val: u8) {
        let (i, j) = var;
        self.0[i as usize][j as usize] = Some(val);
    }

    fn new(_: &()) -> MagicSquare<N> {
        MagicSquare::default()
    }
}

impl<const N: usize> fmt::Display for MagicSquare<N> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for i in 0..N {
            for j in 0..N {
                if let Some(n) = self.0[i][j] {
                    write!(f, "{:3}", n)?;
                } else {
                    write!(f, "  _")?;
                }
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

fn main() {
    println!("Finding all associative 4x4 magic squares.");
    println!();

    let mut solver = Solvomatic::<MagicSquare<4>>::new(());
    solver.config().log_completed = true;

    let mut all_cells = Vec::new();
    for i in 0..4 {
        for j in 0..4 {
            all_cells.push((i, j));
        }
    }

    // Every cell is a number 1..16
    for cell in &all_cells {
        solver.var(*cell, 1..=16);
    }

    // The grid is a permutation of 1..16
    solver.constraint(all_cells.iter().copied(), Permutation::new(1..=16));

    // Each row sums to 34
    for i in 0..4 {
        solver.constraint([(i, 0), (i, 1), (i, 2), (i, 3)], Sum::new(34));
    }
    // Each col sums to 34
    for j in 0..4 {
        solver.constraint([(0, j), (1, j), (2, j), (3, j)], Sum::new(34));
    }
    // So do the diagonals
    solver.constraint([(0, 0), (1, 1), (2, 2), (3, 3)], Sum::new(34));
    solver.constraint([(0, 3), (1, 2), (2, 1), (3, 0)], Sum::new(34));

    // It's an Associative magic square: opposite squares must all have the same sum.
    for i in 0..4 {
        for j in i..4 {
            solver.constraint([(i, j), (3 - i, 3 - j)], Sum::new(17));
        }
    }

    // WLOG, rotate the magic square so that the upper-left cell is least.
    solver.constraint([(0, 0), (0, 3)], Pred::new(|[x, y]| x < y));
    solver.constraint([(0, 0), (3, 0)], Pred::new(|[x, y]| x < y));
    solver.constraint([(0, 0), (3, 3)], Pred::new(|[x, y]| x < y));

    // WLOG, reflect the magic square so that the upper-right cell is less than the lower-left
    // cell.
    solver.constraint([(0, 3), (3, 0)], Pred::new(|[x, y]| x < y));

    solver.solve().unwrap();
    println!("{}", solver.display_table());
}
