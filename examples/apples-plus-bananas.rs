//! From the MIT Mystery Hunt, 2023

use solvomatic::constraints::Pred;
use solvomatic::{Solvomatic, State};
use std::collections::HashMap;
use std::fmt;
use std::fs;

#[derive(Debug, Default)]
struct Apples(HashMap<String, u32>);

impl State for Apples {
    type Var = String;
    type Value = u32;

    fn set(&mut self, fruit: String, count: u32) {
        self.0.insert(fruit, count);
    }
}

impl fmt::Display for Apples {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut fruits = self.0.keys().collect::<Vec<_>>();
        fruits.sort();
        for fruit in fruits {
            let count = self.0.get(fruit).unwrap();
            writeln!(f, "{}: {}", fruit, count)?;
        }
        Ok(())
    }
}

fn is_prime(n: u32) -> bool {
    (2..n).all(|i| n % i != 0)
}

fn main() {
    println!("Solving for fruits...");
    println!();

    let mut solver = Solvomatic::<Apples>::new();
    solver.config().log_completed = true;

    // Read possible fruit PLUs from file, and set a var for each fruit. Example line:
    //
    //     lemon 3617 4053 4033
    let plu_file = fs::read_to_string("examples/apples-plus-bananas-plus.txt").unwrap();
    for line in plu_file.lines() {
        let line = line.trim();
        let mut parts = line.split_whitespace();
        let fruit = parts.next().unwrap().to_owned();
        let plus = parts.map(|plu| plu.parse::<u32>().unwrap());

        solver.var(fruit, plus);
    }

    // Read "equations" (fruit sequences from the puzzle) from file, and add a constraint that each
    // sums to a prime. Example line:
    //
    //     4 blueberry 1 grape 2 peach
    let equation_file = fs::read_to_string("examples/apples-plus-bananas-equations.txt").unwrap();
    for line in equation_file.lines() {
        let line = line.trim();
        let mut parts = line.split_whitespace().collect::<Vec<_>>().into_iter();

        let mut fruits = Vec::new();
        let mut counts = Vec::new();
        while parts.len() > 0 {
            counts.push(parts.next().unwrap().parse::<u32>().unwrap());
            fruits.push(parts.next().unwrap().to_owned());
        }

        // - `fruits` are the parameters
        // - `counts[i]*n` multiplies the value of each fruit by how many times it appears in the quation
        // - `constraint` checks that the sum is prime
        let constraint = Pred::new_with_len(fruits.len(), |array| is_prime(array.iter().sum()));
        solver.mapped_constraint(fruits, move |i, n| counts[i] * n, constraint)
    }

    // So many PLUs for apples, let's just try them all
    solver.var("apple".to_owned(), 4000..5000);

    solver.solve().unwrap();
    println!("{}", solver.table());
}
