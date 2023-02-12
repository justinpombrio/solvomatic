//! Find all two digit numbers AB such that A + B = 3

use solvomatic::constraints::Sum;
use solvomatic::{Solvomatic, State};
use std::fmt;

#[derive(Debug, Default)]
struct Three {
    a: Option<u8>,
    b: Option<u8>,
}

impl State for Three {
    type Var = char;
    type Value = u8;

    fn set(&mut self, var: char, val: u8) {
        match var {
            'A' => self.a = Some(val),
            'B' => self.b = Some(val),
            _ => unreachable!(),
        }
    }
}

impl fmt::Display for Three {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match (self.a, self.b) {
            (None, None) => write!(f, "__"),
            (Some(a), None) => write!(f, "{}_", a),
            (None, Some(b)) => write!(f, "_{}", b),
            (Some(a), Some(b)) => write!(f, "{}{}", a, b),
        }
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
