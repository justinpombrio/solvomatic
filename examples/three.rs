//! Find all two digit numbers AB such that A + B = 3

use solvomatic::constraints::Sum;
use solvomatic::{Solvomatic, State};
use std::collections::HashMap;
use std::fmt;

#[derive(Debug)]
struct Three;

impl State for Three {
    type Var = char;
    type Value = u8;

    fn display(f: &mut String, state: &HashMap<char, u8>) -> fmt::Result {
        use std::fmt::Write;

        for letter in ['A', 'B'] {
            if let Some(digit) = state.get(&letter) {
                write!(f, "{}", digit)?;
            } else {
                write!(f, "_")?;
            }
        }
        Ok(())
    }
}

fn main() {
    let mut solver = Solvomatic::<Three>::new();

    solver.var('A', 1..=9);
    solver.var('B', 0..=9);

    solver.constraint(['A', 'B'], Sum::new(3));

    solver.solve().unwrap();
    println!("{}", solver.table());
}
