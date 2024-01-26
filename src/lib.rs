// TODO:
// - [ ] Better Unsat messages. Right now they often say "one of 0" :-/
// - [ ] Instead of constraints being made of and/or, give them a reference
//       to a table projection, to do what they want with.
// - [ ] Skyscraper constraints
// - [ ] Make constraints into formulas.
// - [ ] Parse visual constraints. E.g.:
//       rule (sum a) = (sum b)
//       | + b +
//       | b a b
//       | + b +
// - [ ] Jake's suggestion: allow forcing distinct words
//       (Needs a rule that accepts a _set_ of seqs of cells to apply to.
//        Keep its Constraint interface the same, just have a fancy constructor,
//        and change parsing in `main.rs` to feed it all patterns.)
// - [ ] Jake's suggestion: allow checking a set of boards by putting them
//       in `initial`. Each is solved independently, then each solution or unsat
//       is listed.
// - [ ] Improve layout parsing error messages. At least give the line number.

// NOTE Solving times on my Yoga laptop for future comparison:
// WordSquare    -  4s   ->  1s
// MagicHexagon  - 14s   ->  7s
// JigsawSudoku2 -  1s   -> 170ms
// JigsawSudoku9 -  7s   ->  1s
// Palindrome    -  2s   ->  1s
// MagicSquare   - 400ms -> 300ms

//! Some puzzles require a spark of insight, a sudden recognition, or a clever twist of thought.
//!
//!  For all the others, there's solv-o-matic.
//!
//!  _Solv-o-matic is pre-alpha software. It's likely to contain bugs, and its
//!  interface may change dramatically at any moment._
//!
//! Solv-o-matic can be used either as an application invoked on a text file, or as a library.
//! For docs on its use as an application, see the README. For docs on using it as a library,
//! keep reading.
//!
//! ## Solving a Sudoku
//!
//! Let's see how to solve a (supposedly) hard Sudoku in code.
//!
//! First some imports:
//!
//! ```
//! use solvomatic::constraints::{Permutation, Pred};
//! use solvomatic::{Solvomatic, State};
//! use std::fmt;
//! ```
//!
//! Now we declare a name for the puzzle state. Let's use a `u8` for each digit in
//! the Sudoku. But actually an `Option<u8>`, because not all
//! values might be known. So we'll use a 9x9 matrix of `Option<u8>` for the board:
//!
//! ```
//! #[derive(Debug, Default)]
//! struct Sudoku([[Option<u8>; 9]; 9]);
//! ```
//!
//! Next we declare how `Sudoku` implements a puzzle `State`. This requires:
//!
//! - The `Var` type, here `(usize, usize)` as a (row, col) index to identify a
//!   cell.
//! - The `Value` type, here a `u8` for each cell.
//! - A `set` function for setting the value of one cell.
//! - A `new` method that constructs a blank sudoku. _Some_ puzzles require some
//!   extra data to create a blank board, so it gets a `MetaData` value. We don't
//!   need this extra data though, so we just declare `type MetaData = ()`.
//!
//! ```
//! # use std::fmt;
//! # use solvomatic::{Solvomatic, State};
//! # #[derive(Debug, Default)]
//! # struct Sudoku([[Option<u8>; 9]; 9]);
//! impl State for Sudoku {
//!     type Var = (usize, usize);
//!     type Value = u8;
//!     type MetaData = ();
//!
//!     fn set(&mut self, var: (usize, usize), val: u8) {
//!         let (i, j) = var;
//!         self.0[i - 1][j - 1] = Some(val);
//!     }
//!
//!     fn new(_metadata: &()) -> Sudoku {
//!         Sudoku::default()
//!     }
//! }
//! # impl fmt::Display for Sudoku {
//! #     fn fmt(&self, _f: &mut fmt::Formatter) -> fmt::Result {
//! #         unimplemented!()
//! #     }
//! # }
//! ```
//!
//! The last thing that our `State` needs is to implement `Display`, so that you can
//! see the board. We'll print `_` for unknown cells:
//!
//! ```
//! # use std::fmt;
//! # use solvomatic::{Solvomatic, State};
//! # #[derive(Debug, Default)]
//! # struct Sudoku([[Option<u8>; 9]; 9]);
//! # impl State for Sudoku {
//! #     type Var = (usize, usize);
//! #     type Value = u8;
//! #     type MetaData = ();
//! #     fn set(&mut self, var: (usize, usize), val: u8) {
//! #         let (i, j) = var;
//! #         self.0[i - 1][j - 1] = Some(val);
//! #     }
//! #     fn new(_metadata: &()) -> Sudoku {
//! #         Sudoku::default()
//! #     }
//! # }
//! impl fmt::Display for Sudoku {
//!     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//!         writeln!(f, "+---+---+---+")?;
//!         for (i, row) in self.0.iter().enumerate() {
//!             write!(f, "|")?;
//!             for (j, cell) in row.iter().enumerate() {
//!                 if let Some(n) = cell {
//!                     write!(f, "{:1}", n)?;
//!                 } else {
//!                     write!(f, "_")?;
//!                 }
//!                 if j % 3 == 2 {
//!                     write!(f, "|")?;
//!                 }
//!             }
//!             writeln!(f)?;
//!             if i % 3 == 2 {
//!                 writeln!(f, "+---+---+---+")?;
//!             }
//!         }
//!         Ok(())
//!     }
//! }
//! ```
//!
//! Another requirement for `State` is that it implements `Debug` and `Default`. We
//! `derive`d those above.
//!
//! Now let's make a Sudoku solver!
//!
//! ```
//! # use std::fmt;
//! # use solvomatic::{Solvomatic, State};
//! # use solvomatic::constraints::{Permutation, Pred};
//! # #[derive(Debug, Default)]
//! # struct Sudoku([[Option<u8>; 9]; 9]);
//! # impl State for Sudoku {
//! #     type Var = (usize, usize);
//! #     type Value = u8;
//! #     type MetaData = ();
//! #     fn set(&mut self, var: (usize, usize), val: u8) {
//! #         let (i, j) = var;
//! #         self.0[i - 1][j - 1] = Some(val);
//! #     }
//! #     fn new(_metadata: &()) -> Sudoku {
//! #         Sudoku::default()
//! #     }
//! # }
//! # impl fmt::Display for Sudoku {
//! #     fn fmt(&self, _f: &mut fmt::Formatter) -> fmt::Result {
//! #         unimplemented!()
//! #     }
//! # }
//! let mut solver = Solvomatic::<Sudoku>::new(());
//!
//! // There are 81 cells, identified by (i, j). Each cell is a number 1..9.
//! for i in 1..=9 {
//!     for j in 1..=9 {
//!         solver.var((i, j), 1..=9);
//!     }
//! }
//!
//! // Each row is a permutation of 1..9
//! for i in 1..=9 {
//!     let row: [(usize, usize); 9] = std::array::from_fn(|j| (i, j + 1));
//!     solver.constraint(row, Permutation::new(1..=9));
//! }
//!
//! // Each col is a permutation of 1..9
//! for j in 1..=9 {
//!     let col: [(usize, usize); 9] = std::array::from_fn(|i| (i + 1, j));
//!     solver.constraint(col, Permutation::new(1..=9));
//! }
//!
//! // Each 3x3 block is a permutation of 1..9
//! for block_i in 0..3 {
//!     for block_j in 0..3 {
//!         let mut block_cells = Vec::new();
//!         for i in 1..=3 {
//!             for j in 1..=3 {
//!                 block_cells.push((block_i * 3 + i, block_j * 3 + j));
//!             }
//!         }
//!         solver.constraint(block_cells, Permutation::new(1..=9));
//!     }
//! }
//! ```
//!
//! Now set the constraints specific to this puzzle:
//!
//! ```
//! # use std::fmt;
//! # use solvomatic::{Solvomatic, State};
//! # use solvomatic::constraints::{Permutation, Pred};
//! # #[derive(Debug, Default)]
//! # struct Sudoku([[Option<u8>; 9]; 9]);
//! # impl State for Sudoku {
//! #     type Var = (usize, usize);
//! #     type Value = u8;
//! #     type MetaData = ();
//! #     fn set(&mut self, var: (usize, usize), val: u8) {
//! #         let (i, j) = var;
//! #         self.0[i - 1][j - 1] = Some(val);
//! #     }
//! #     fn new(_metadata: &()) -> Sudoku {
//! #         Sudoku::default()
//! #     }
//! # }
//! # impl fmt::Display for Sudoku {
//! #     fn fmt(&self, _f: &mut fmt::Formatter) -> fmt::Result {
//! #         unimplemented!()
//! #     }
//! # }
//! # let mut solver = Solvomatic::<Sudoku>::new(());
//! // The starting config for this particular sudoku
//! // (row, col, num)
//! let prefilled: &[(usize, usize, u8)] = &[
//!     (1, 3, 5), (2, 1, 6), (1, 4, 9), (2, 5, 5),
//!     (2, 6, 3), (3, 4, 2), (1, 7, 4), (2, 7, 8),
//!     (3, 9, 3), (4, 5, 9), (5, 1, 2), (5, 8, 4),
//!     (6, 3, 4), (6, 5, 8), (6, 6, 5), (6, 9, 1),
//!     (7, 3, 2), (7, 5, 4), (7, 6, 1), (7, 9, 8),
//!     (8, 2, 7), (8, 7, 6), (9, 4, 3),
//! ];
//! for (i, j, n) in prefilled {
//!     solver.constraint([(*i, *j)], Pred::new(|[x]| *x == *n));
//! }
//! ```
//!
//! Finally we tell it to solve! If `solve()` fails, it will produce an
//! `Unsatisfiable` error. Otherwise, we print the answers in `solver.display_table()`.
//!
//! ```
//! # use std::fmt;
//! # use solvomatic::{Solvomatic, State};
//! # use solvomatic::constraints::{Permutation, Pred};
//! # #[derive(Debug, Default)]
//! # struct Sudoku([[Option<u8>; 9]; 9]);
//! # impl State for Sudoku {
//! #     type Var = (usize, usize);
//! #     type Value = u8;
//! #     type MetaData = ();
//! #     fn set(&mut self, var: (usize, usize), val: u8) {
//! #         let (i, j) = var;
//! #         self.0[i - 1][j - 1] = Some(val);
//! #     }
//! #     fn new(_metadata: &()) -> Sudoku {
//! #         Sudoku::default()
//! #     }
//! # }
//! # impl fmt::Display for Sudoku {
//! #     fn fmt(&self, _f: &mut fmt::Formatter) -> fmt::Result {
//! #         unimplemented!()
//! #     }
//! # }
//! # let mut solver = Solvomatic::<Sudoku>::new(());
//! match solver.solve() {
//!     Ok(()) => (),
//!     Err(err) => panic!("{}", err),
//! }
//! println!("{}", solver.display_table());
//! ```
//!
//! And it spits out the possible solutions:
//!
//! ```text
//! solvomatic> cargo run --example sudoku
//!    Compiling solvomatic v0.3.0 (/home/justin/git/solvomatic)
//!     Finished release [optimized] target(s) in 2.32s
//!      Running `target/release/examples/sudoku`
//! Solving a hard sudoku.
//!
//! Step  0: size =  139 possibilities = 160489808068608
//!   elapsed:   347ms
//! time: 362ms
//! State is one of 1:
//!     +---+---+---+
//!     |325|918|467|
//!     |649|753|812|
//!     |817|264|593|
//!     +---+---+---+
//!     |531|492|786|
//!     |286|137|945|
//!     |794|685|231|
//!     +---+---+---+
//!     |962|541|378|
//!     |173|829|654|
//!     |458|376|129|
//!     +---+---+---+
//! ```

// I'm all for warning against overly complex types, but Clippy is warning on some
// types that aren't very complicated.
#![allow(clippy::type_complexity)]
// No strong feelings on this but it's a reasonable way to write things and it's
// how the code currently works.
#![allow(clippy::result_unit_err)]

mod guessing_solver;
mod state;
mod table;

use constraints::Constraint;
use std::fmt;
use std::time::Instant;

pub mod constraints;

pub use guessing_solver::GuessingSolver;
pub use state::State;
pub use table::Table;

/// Solves puzzles in much the same way that hitting them with a brick doesn't.
pub struct Solvomatic<S: State> {
    table: Table<S>,
    constraints: Vec<DynConstraint<S>>,
    config: Config,
    metadata: S::MetaData,
}

/// The problem was over constrained! Contained is a snapshot of the Table just before a constraint
/// was applied that shrunk that Table's number of possibilities to zero, together with information
/// about that constraint.
#[derive(Debug, Clone)]
pub struct Unsatisfiable<S: State> {
    pub table: Table<S>,
    pub header: Vec<S::Var>,
    pub constraint: String,
    metadata: S::MetaData,
}

struct DynConstraint<S: State> {
    name: String,
    params: Vec<S::Var>,
    apply: Box<dyn Fn(&mut Table<S>) -> Result<bool, ()> + Send + Sync + 'static>,
    done: bool,
}

// When running `main`, this is loaded from command line args.
// See `Config` in `main.rs`.
/// Configuration options. Set these using `Solvomatic.config()`.
#[derive(Debug, Clone, Default)]
pub struct Config {
    /// Log after each step that's taken
    pub log_steps: bool,
    /// Log the list of contraints before solving
    pub log_constraints: bool,
    /// Log when a constraint is completed
    pub log_completed: bool,
    /// Log how long each step took
    pub log_elapsed: bool,
    /// Log intermediate states (these can be very large!)
    pub log_states: bool,
}

impl<S: State> Solvomatic<S> {
    /// Construct an empty solver. Call `var()` and `constraint()` to give it variables and
    /// constraints, then `solve()` to solve for them.
    pub fn new(metadata: S::MetaData) -> Solvomatic<S> {
        Solvomatic {
            table: Table::new(),
            constraints: Vec::new(),
            config: Config::default(),
            metadata,
        }
    }

    pub fn config(&mut self) -> &mut Config {
        &mut self.config
    }

    /// Add a new variable, with a set of possible values.
    pub fn var(&mut self, x: S::Var, values: impl IntoIterator<Item = S::Value>) {
        self.table.add_column(x, values);
    }

    /// Add the requirement that the variables `params` must obey `constraint`.
    pub fn constraint<C: Constraint<S::Value>>(
        &mut self,
        params: impl IntoIterator<Item = S::Var>,
        constraint: C,
    ) {
        self.mapped_constraint(params, |_, v| v, constraint)
    }

    /// Add the requirement that the variables `params`, after being `map`ed, must obey
    /// `constraint`.
    pub fn mapped_constraint<N, C: Constraint<N>>(
        &mut self,
        params: impl IntoIterator<Item = S::Var>,
        map: impl Fn(usize, S::Value) -> N + Send + Sync + 'static,
        constraint: C,
    ) {
        let name = C::NAME.to_owned();
        let params = params.into_iter().collect::<Vec<_>>();

        if self.config.log_constraints {
            eprintln!("Constraint {} on {:?} = {:?}", name, params, constraint);
        }

        let params_copy = params.clone();
        let apply = Box::new(move |table: &mut Table<S>| {
            table.apply_constraint(&params_copy, &map, &constraint)
        });
        self.constraints.push(DynConstraint {
            name,
            params,
            apply,
            done: false,
        });
    }

    /// Solves the constraints! Returns `Err(Unsatisfiable)` if it discovers that the constraints
    /// are not, in fact, possible to satisfy. Otherwise, call `.table()` to see the solution(s).
    pub fn solve(&mut self) -> Result<(), Unsatisfiable<S>> {
        let start_time = Instant::now();

        self.table = self.apply_constraints(self.table.clone())?;
        while self.table.num_partitions() > 1 && self.table.possibilities() > 1.0 {
            self.step()?;
        }
        self.table.merge_constants();

        if self.config.log_steps {
            eprintln!("Total time: {}ms", start_time.elapsed().as_millis());
        }

        Ok(())
    }

    /// Apply one step of solving. It's important to `apply_constraints()` _before_ the first step
    /// though!
    fn step(&mut self) -> Result<(), Unsatisfiable<S>> {
        use rayon::prelude::*;

        let start_time = Instant::now();

        // Mark completed constraints as done
        self.mark_completed_constraints();

        // Merge all constant partitions together
        self.table.merge_constants();

        if self.config.log_states {
            eprintln!("{}", self.table.display(&self.metadata));
        }
        if self.config.log_steps {
            eprintln!(
                "\nNumber of partitions = {:2}, table size = {:4}, possibilities = {}",
                self.table.num_partitions(),
                self.table.size(),
                self.table.possibilities(),
            );
        }

        // Consider merging all combinations of two Sections of the table
        if self.table.num_partitions() > 1 {
            let mut options = Vec::new();
            for i in 0..self.table.num_partitions() - 1 {
                for j in i + 1..self.table.num_partitions() {
                    options.push((i, j));
                }
            }
            let result = options
                .par_iter()
                .map(&|(i, j): &(usize, usize)| {
                    let mut new_table = self.table.clone();
                    new_table.merge(*i, *j);
                    self.apply_constraints(new_table)
                })
                .reduce_with(|a, b| match (a, b) {
                    (Err(err), _) | (_, Err(err)) => Err(err),
                    (Ok(table_a), Ok(table_b)) => {
                        if table_a.cost() <= table_b.cost() {
                            Ok(table_a)
                        } else {
                            Ok(table_b)
                        }
                    }
                });

            self.table = result.unwrap()?;
        }

        // Log how long it took
        if self.config.log_elapsed {
            let elapsed_time = start_time.elapsed().as_millis();
            eprintln!("  elapsed: {:5?}ms", elapsed_time);
        }

        Ok(())
    }

    /// Repeatedly apply all constraints until that stops having any effect.
    fn apply_constraints(&self, mut table: Table<S>) -> Result<Table<S>, Unsatisfiable<S>> {
        let mut last_size = table.size() + 1;
        while table.size() < last_size {
            last_size = table.size();
            for constraint in &self.constraints {
                if constraint.done {
                    continue;
                }
                match (constraint.apply)(&mut table) {
                    Ok(_) => (),
                    Err(()) => {
                        return Err(Unsatisfiable {
                            table,
                            constraint: constraint.name.clone(),
                            header: constraint.params.clone(),
                            metadata: self.metadata.clone(),
                        })
                    }
                }
            }
        }
        Ok(table)
    }

    /// Mark constraints that will _always_ hold as done.
    fn mark_completed_constraints(&mut self) {
        for constraint in &mut self.constraints {
            if constraint.done {
                continue;
            }
            if (constraint.apply)(&mut self.table.clone()) == Ok(true) {
                if self.config.log_completed {
                    println!(
                        "  completed constraint {} {:?}",
                        constraint.name, constraint.params
                    );
                }
                constraint.done = true;
            }
        }
    }

    /// The current table of possibilities.
    pub fn table(&self) -> &Table<S> {
        &self.table
    }

    pub fn display_table(&self) -> impl fmt::Display + '_ {
        self.table.display(&self.metadata)
    }
}

impl<S: State> fmt::Display for Unsatisfiable<S> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "UNSATISFIABLE!")?;
        writeln!(f, "{}", self.table.display(&self.metadata))?;
        write!(f, "Constraint {} on [", self.constraint)?;
        for variable in &self.header {
            write!(f, "{:?} ", variable)?;
        }
        writeln!(f, "]")?;
        write!(f, "is unsatisfiable")
    }
}
