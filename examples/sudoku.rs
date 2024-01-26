//! Solve a hard Sudoku

use solvomatic::constraints::{Permutation, Pred};
use solvomatic::{Solvomatic, State};
use std::fmt;

#[derive(Debug, Default)]
struct Sudoku([[Option<u8>; 9]; 9]);

impl State for Sudoku {
    type Var = (usize, usize);
    type Value = u8;
    type MetaData = ();

    fn set(&mut self, var: (usize, usize), val: u8) {
        let (i, j) = var;
        self.0[i - 1][j - 1] = Some(val);
    }

    fn new(_: &()) -> Sudoku {
        Sudoku::default()
    }
}

impl fmt::Display for Sudoku {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "+---+---+---+")?;
        for (i, row) in self.0.iter().enumerate() {
            write!(f, "|")?;
            for (j, cell) in row.iter().enumerate() {
                if let Some(n) = cell {
                    write!(f, "{:1}", n)?;
                } else {
                    write!(f, "_")?;
                }
                if j % 3 == 2 {
                    write!(f, "|")?;
                }
            }
            writeln!(f)?;
            if i % 3 == 2 {
                writeln!(f, "+---+---+---+")?;
            }
        }
        Ok(())
    }
}

fn main() {
    println!("Solving a hard sudoku.");
    println!();

    let mut solver = Solvomatic::<Sudoku>::new(());
    solver.config().log_elapsed = true;

    // There are 81 cells. Each cell is a number 1..9
    for i in 1..=9 {
        for j in 1..=9 {
            solver.var((i, j), 1..=9);
        }
    }

    // Each row is a permutation of 1..9
    for i in 1..=9 {
        let row: [(usize, usize); 9] = std::array::from_fn(|j| (i, j + 1));
        solver.constraint(row, Permutation::new(1..=9));
    }

    // Each col is a permutation of 1..9
    for j in 1..=9 {
        let col: [(usize, usize); 9] = std::array::from_fn(|i| (i + 1, j));
        solver.constraint(col, Permutation::new(1..=9));
    }

    // Each 3x3 block is a permutation of 1..9
    for block_i in 0..3 {
        for block_j in 0..3 {
            let mut block_cells = Vec::new();
            for i in 1..=3 {
                for j in 1..=3 {
                    block_cells.push((block_i * 3 + i, block_j * 3 + j));
                }
            }
            solver.constraint(block_cells, Permutation::new(1..=9));
        }
    }

    // The starting config for this particular sudoku (row, col, num)
    let prefilled: &[(usize, usize, u8)] = &[
        (1, 3, 5),
        (2, 1, 6),
        (1, 4, 9),
        (2, 5, 5),
        (2, 6, 3),
        (3, 4, 2),
        (1, 7, 4),
        (2, 7, 8),
        (3, 9, 3),
        (4, 5, 9),
        (5, 1, 2),
        (5, 8, 4),
        (6, 3, 4),
        (6, 5, 8),
        (6, 6, 5),
        (6, 9, 1),
        (7, 3, 2),
        (7, 5, 4),
        (7, 6, 1),
        (7, 9, 8),
        (8, 2, 7),
        (8, 7, 6),
        (9, 4, 3),
    ];
    for (i, j, n) in prefilled {
        solver.constraint([(*i, *j)], Pred::new(|[x]| *x == *n));
    }

    let solutions = solver.solve();
    if solutions.0.is_empty() {
        println!("No solutions");
    } else {
        println!("{}", solutions);
    }
}
