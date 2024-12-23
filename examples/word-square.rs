//! Find all 4x4 word squares whose diagonals are vowels

use solvomatic::constraints::{Pred, Seq};
use solvomatic::{Solvomatic, State};
use std::fmt;

/// An NxN word square
#[derive(Debug)]
struct WordSquare<const N: usize>([[Option<char>; N]; N]);

impl<const N: usize> Default for WordSquare<N> {
    fn default() -> WordSquare<N> {
        WordSquare([[None; N]; N])
    }
}

impl<const N: usize> State for WordSquare<N> {
    type Var = (usize, usize);
    type Value = char;
    type MetaData = ();

    fn set(&mut self, var: (usize, usize), letter: char) {
        let (i, j) = var;
        self.0[i][j] = Some(letter);
    }

    fn new(_: &()) -> WordSquare<N> {
        WordSquare::default()
    }
}

impl<const N: usize> fmt::Display for WordSquare<N> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for row in &self.0 {
            for cell in row {
                if let Some(letter) = cell {
                    write!(f, "{}", letter.to_uppercase())?;
                } else {
                    write!(f, "_")?;
                }
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

fn main() {
    println!("Finding all 4x4 word squares whose diagonals are vowels.");
    println!();

    let mut solver = Solvomatic::<WordSquare<4>>::new(());
    solver.config().log_completed = true;

    let mut all_cells = Vec::new();
    for i in 0..4 {
        for j in 0..4 {
            all_cells.push((i, j));
        }
    }

    // Every cell is a letter, and a letter along a diagonal must be a vowel.
    let diagonals = [
        (0, 0),
        (1, 1),
        (2, 2),
        (3, 3),
        (0, 3),
        (1, 2),
        (2, 1),
        (3, 0),
    ];
    for cell in &all_cells {
        if diagonals.contains(cell) {
            solver.var(*cell, ['a', 'e', 'i', 'o', 'u']);
        } else {
            solver.var(*cell, 'a'..='z');
        }
    }

    // Every row and col forms a word
    let word_of_len_4 = Seq::word_list_file("/usr/share/dict/words", 4).unwrap();
    for i in 0..4 {
        solver.constraint([(i, 0), (i, 1), (i, 2), (i, 3)], word_of_len_4.clone());
    }
    for j in 0..4 {
        solver.constraint([(0, j), (1, j), (2, j), (3, j)], word_of_len_4.clone());
    }

    // WLOG, reflect the word square so that the upper-right cell is less than the lower-left
    // cell.
    solver.constraint([(0, 3), (3, 0)], Pred::new(|[x, y]| x < y));

    let solutions = solver.solve();
    if solutions.0.is_empty() {
        println!("No solutions");
    } else {
        println!("{}", solutions);
    }
}
