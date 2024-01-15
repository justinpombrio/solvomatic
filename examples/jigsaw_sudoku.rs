//! Solver for the Sudokus in the MIT Mystery Hunt 2024 puzzle 'Jigsaw Sudoku'.
//!
//! https://mythstoryhunt.world/puzzles/jigsaw-sudoku
//!
//! Usage: cargo run --release --example jigsaw_sudoku examples/input/sudoku_2.txt examples/input/regions_2.txt

use solvomatic::constraints::{Count, Permutation, Pred};
use solvomatic::{Solvomatic, State};
use std::collections::HashMap;
use std::env;
use std::fmt;
use std::fs;

#[derive(Debug, Default)]
struct Sudoku([[Option<u8>; 9]; 9]);

impl State for Sudoku {
    type Var = (usize, usize);
    type Value = u8;

    fn set(&mut self, var: (usize, usize), val: u8) {
        let (i, j) = var;
        self.0[i - 1][j - 1] = Some(val);
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

/// Invoke a callback on each `(character, row, col)` triple from a string.
/// Newlines are skipped (but influence the row).
fn callback_by_row_col(contents: &str, mut callback: impl FnMut(char, usize, usize)) {
    let (mut row, mut col) = (0, 0);
    for ch in contents.chars() {
        if ch == '\n' {
            row += 1;
            col = 0;
        } else {
            callback(ch, row, col);
            col += 1;
        }
    }
}

fn main() {
    println!("Solving sudoku with additional regions constraint.");
    println!();

    // Read command line args: paths to sudoku and regions file
    let args: Vec<String> = env::args().collect();
    assert_eq!(args.len(), 3);
    let sudoku_filename = &args[1];
    let regions_filename = &args[2];

    let mut solver = Solvomatic::<Sudoku>::new();
    solver.config().log_elapsed = true;
    solver.config().log_states = true;

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
    let sudoku_string = fs::read_to_string(sudoku_filename).unwrap();
    callback_by_row_col(&sudoku_string, |ch, row, col| {
        use std::str::FromStr;
        if ch != '.' {
            let digit = u8::from_str(&format!("{}", ch)).unwrap();
            solver.constraint([(row + 1, col + 1)], Pred::new(move |[x]| *x == digit));
        }
    });

    // In each "region" (given by char in region file), each number must appear
    // at least once and at most twice.
    let regions_string = fs::read_to_string(regions_filename).unwrap();
    let mut regions = HashMap::new();
    callback_by_row_col(&regions_string, |ch, row, col| {
        regions
            .entry(ch)
            .or_insert_with(|| Vec::new())
            .push((row + 1, col + 1));
    });
    for (_, region) in regions.into_iter() {
        solver.constraint(region, Count::new((1..=9).map(|n| (n, 1, 2))));
    }

    match solver.solve() {
        Ok(()) => (),
        Err(err) => {
            println!("{}", err);
            panic!("unsat");
        }
    }
    println!("{}", solver.table());
}
