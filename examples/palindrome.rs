//! Find all six letter palindromes in /usr/share/dict/words

use solvomatic::constraints::{Pred, Seq};
use solvomatic::{Solvomatic, State};
use std::fmt;

#[derive(Debug, Default)]
struct Palindrome([Option<char>; 6]);

impl State for Palindrome {
    type Var = usize;
    type Value = char;
    type MetaData = ();

    fn set(&mut self, var: usize, val: char) {
        self.0[var] = Some(val);
    }

    fn new(_: &()) -> Palindrome {
        Palindrome::default()
    }
}

impl fmt::Display for Palindrome {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for letter in &self.0 {
            if let Some(letter) = letter {
                write!(f, "{}", letter)?;
            } else {
                write!(f, "_")?;
            }
        }
        Ok(())
    }
}

fn main() {
    println!("Finding all six letter palindromes in /usr/share/dict/words");
    println!();

    let mut solver = Solvomatic::<Palindrome>::new(());
    solver.config().log_completed = true;

    // Every cell is a letter
    solver.var(0, 'a'..='z');
    solver.var(1, 'a'..='z');
    solver.var(2, 'a'..='z');
    solver.var(3, 'a'..='z');
    solver.var(4, 'a'..='z');
    solver.var(5, 'a'..='z');

    // The whole thing is a word
    let word_of_len_6 = Seq::word_list_file("/usr/share/dict/words", 6).unwrap();
    solver.constraint([0, 1, 2, 3, 4, 5], word_of_len_6);

    // It's a palindrome
    solver.constraint([0, 5], Pred::new(|[a, b]| *a == *b));
    solver.constraint([1, 4], Pred::new(|[a, b]| *a == *b));
    solver.constraint([2, 3], Pred::new(|[a, b]| *a == *b));

    let solutions = solver.solve();
    if solutions.0.is_empty() {
        println!("No solutions");
    } else {
        println!("{}", solutions);
    }
}
