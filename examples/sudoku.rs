//! Find a hard Sudoku

use solvomatic::constraints::{Bag, Pred};
use solvomatic::{Solvomatic, State};
use std::collections::HashMap;
use std::fmt;

#[derive(Debug)]
struct Sudoku;

impl State for Sudoku {
    type Var = (i8, i8);
    type Value = u8;

    fn display(f: &mut String, state: &HashMap<(i8, i8), u8>) -> fmt::Result {
        use std::fmt::Write;

        fn show_cell(f: &mut String, i: i8, j: i8, state: &HashMap<(i8, i8), u8>) -> fmt::Result {
            if let Some(n) = state.get(&(i, j)) {
                write!(f, "{:1}", n)
            } else {
                write!(f, "_")
            }
        }

        writeln!(f, "+---+---+---+")?;
        for i in 1..=9 {
            write!(f, "|")?;
            for j in 1..=9 {
                show_cell(f, i, j, state)?;
                if j % 3 == 0 {
                    write!(f, "|")?;
                }
            }
            writeln!(f)?;
            if i % 3 == 0 {
                writeln!(f, "+---+---+---+")?;
            }
        }
        Ok(())
    }
}

fn main() {
    println!("Solving a hard sudoku.");
    println!();

    let mut solver = Solvomatic::<Sudoku>::new();
    solver.config().log_elapsed = true;

    // There are 81 cells
    let mut all_cells = Vec::new();
    for i in 1..=9 {
        for j in 1..=9 {
            all_cells.push((i, j));
        }
    }

    // Each cell is a number 1..9
    for cell in &all_cells {
        solver.var(*cell, 1..=9);
    }

    // Each row is a permutation of 1..9
    for i in 1..=9 {
        let row: [(i8, i8); 9] = std::array::from_fn(|j| (i, j as i8 + 1));
        solver.constraint(row, Bag::new(1..=9));
    }

    // Each col is a permutation of 1..9
    for j in 1..=9 {
        let col: [(i8, i8); 9] = std::array::from_fn(|i| (i as i8 + 1, j));
        solver.constraint(col, Bag::new(1..=9));
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
            solver.constraint(block_cells, Bag::new(1..=9));
        }
    }

    // The starting config for this particular sudoku
    // (row, col, num)
    let fixed: &[(i8, i8, u8)] = &[
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
    for (i, j, n) in fixed {
        solver.constraint([(*i, *j)], Pred::new(|[x]| *x == *n));
    }

    solver.solve().unwrap();
    println!("{}", solver.table());
}
