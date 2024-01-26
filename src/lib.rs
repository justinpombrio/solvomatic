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
//! #     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//! #         writeln!(f, "+---+---+---+")?;
//! #         for (i, row) in self.0.iter().enumerate() {
//! #             write!(f, "|")?;
//! #             for (j, cell) in row.iter().enumerate() {
//! #                 if let Some(n) = cell {
//! #                     write!(f, "{:1}", n)?;
//! #                 } else {
//! #                     write!(f, "_")?;
//! #                 }
//! #                 if j % 3 == 2 {
//! #                     write!(f, "|")?;
//! #                 }
//! #             }
//! #             writeln!(f)?;
//! #             if i % 3 == 2 {
//! #                 writeln!(f, "+---+---+---+")?;
//! #             }
//! #         }
//! #         Ok(())
//! #     }
//! # }
//! # let mut solver = Solvomatic::<Sudoku>::new(());
//! let solutions = solver.solve();
//! if solutions.0.is_empty() {
//!     println!("No solutions");
//! } else {
//!     println!("{}", solutions);
//! }
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

mod state;

pub mod constraints;
pub use state::{State, StateSet};

use constraints::{Constraint, YesNoMaybe};
use std::fmt;
use std::mem;
use std::time::Instant;

/************************
 *     Table            *
 ************************/

type VarIndex = usize;
type EntryIndex = usize;

pub struct Table<S: State> {
    /// VarIndex -> Var
    vars: Vec<S::Var>,
    /// VarIndex -> set of Value
    entries: Vec<Vec<S::Value>>,
}

// #derive doesn't work here; it inappropriately requires S: Clone
impl<S: State> Clone for Table<S> {
    fn clone(&self) -> Self {
        Table {
            vars: self.vars.clone(),
            entries: self.entries.clone(),
        }
    }
}

impl<S: State> Table<S> {
    fn new() -> Table<S> {
        Table {
            vars: Vec::new(),
            entries: Vec::new(),
        }
    }

    pub fn add_column(&mut self, var: S::Var, values: impl IntoIterator<Item = S::Value>) {
        let vals = values.into_iter().collect::<Vec<_>>();
        if vals.is_empty() {
            panic!("Empty range given for variable {:?}", var);
        }
        self.vars.push(var.clone());
        self.entries.push(vals);
    }

    fn size(&self) -> usize {
        let mut size = 0;
        for values in &self.entries {
            size += values.len();
        }
        size
    }

    fn var_guessing_score(&self, var: VarIndex) -> i32 {
        match self.entries[var].len() {
            1 => 0,
            n => 1000_000 - n as i32,
        }
    }

    fn make_guess(&mut self, var: VarIndex, guess: EntryIndex) {
        self.entries[var] = vec![self.entries[var].swap_remove(guess)];
    }

    fn guess(self) -> Vec<Table<S>> {
        let var_to_guess = (0..self.entries.len())
            .max_by_key(|i| self.var_guessing_score(*i))
            .unwrap_or(0);
        let num_guesses = self.entries[var_to_guess].len();
        (0..num_guesses)
            .map(|guess| {
                let mut table = self.clone();
                table.make_guess(var_to_guess, guess);
                table
            })
            .collect::<Vec<_>>()
    }

    fn eval_constraint_for_param<C: Constraint<S::Value>>(
        &self,
        constraint: &C,
        param_index: usize,
        var: S::Var,
        assume: Option<(VarIndex, EntryIndex)>,
    ) -> C::Set {
        let var_index = self.vars.iter().position(|v| *v == var).unwrap();
        if let Some((assumed_var, assumed_entry)) = assume {
            if assumed_var == var_index {
                return constraint
                    .singleton(param_index, self.entries[var_index][assumed_entry].clone());
            }
        }

        let mut values_iter = self.entries[var_index].iter();
        let mut set = constraint.singleton(param_index, values_iter.next().unwrap().clone());
        for value in values_iter {
            set = constraint.or(set, constraint.singleton(param_index, value.clone()));
        }
        set
    }

    fn eval_constraint_for_all<C: Constraint<S::Value>>(
        &self,
        constraint: &C,
        params: &Vec<S::Var>,
        param_index: usize,
    ) -> Vec<YesNoMaybe> {
        let var = &params[param_index];
        let var_index = self.vars.iter().position(|v| v == var).unwrap();
        let values_iter = self.entries[var_index].iter().cloned();

        if params.len() == 1 {
            assert_eq!(param_index, 0);
            return values_iter
                .map(|value| constraint.check(constraint.singleton(param_index, value)))
                .collect();
        }

        let mut params_iter = params.iter().enumerate().filter(|(i, _)| *i != param_index);
        let (first_param_index, first_var) = params_iter.next().unwrap();
        let mut set =
            self.eval_constraint_for_param(constraint, first_param_index, first_var.clone(), None);
        for (param_index, var) in params_iter {
            set = constraint.and(
                set,
                self.eval_constraint_for_param(constraint, param_index, var.clone(), None),
            );
        }

        values_iter
            .map(|value| {
                constraint
                    .check(constraint.and(set.clone(), constraint.singleton(param_index, value)))
            })
            .collect()
    }

    // TODO
    #[allow(unused)]
    fn eval_constraint<C: Constraint<S::Value>>(
        &self,
        constraint: &C,
        params: &Vec<S::Var>,
        assume: Option<(VarIndex, EntryIndex)>,
    ) -> YesNoMaybe {
        if let Some((var, _)) = assume {
            if !params.contains(&self.vars[var]) {
                // The relevant var isn't one of our params!
                return YesNoMaybe::Yes;
            }
        }
        let mut params_iter = params.iter().enumerate();
        let (first_param_index, first_var) = params_iter.next().unwrap();
        let mut set = self.eval_constraint_for_param(
            constraint,
            first_param_index,
            first_var.clone(),
            assume,
        );
        for (param_index, var) in params_iter {
            set = constraint.and(
                set,
                self.eval_constraint_for_param(constraint, param_index, var.clone(), assume),
            );
        }
        constraint.check(set)
    }

    fn is_solved(&self) -> bool {
        for values in &self.entries {
            if values.len() > 1 {
                return false;
            }
        }
        true
    }

    fn into_state(&self, metadata: &S::MetaData) -> S {
        let mut solution = S::new(metadata);
        for (var, values) in self.vars.iter().zip(self.entries.iter()) {
            if values.len() == 1 {
                solution.set(var.clone(), values[0].clone());
            }
        }
        solution
    }
}

impl<S: State> fmt::Display for Table<S> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "Table:")?;
        for (i, var) in self.vars.iter().enumerate() {
            write!(f, "    {:?}:", var)?;
            for val in &self.entries[i] {
                write!(f, " {:?}", val)?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

/************************
 *     DynConstraint    *
 ************************/

struct DynConstraint<S: State> {
    // TODO
    #[allow(unused)]
    name: String,
    params: Vec<S::Var>,
    eval: Box<dyn Fn(&Table<S>, usize) -> Vec<YesNoMaybe> + Send + Sync + 'static>,
}

impl<S: State> DynConstraint<S> {
    fn new<C: Constraint<S::Value>>(
        params: impl IntoIterator<Item = S::Var>,
        constraint: C,
    ) -> DynConstraint<S> {
        let name = C::NAME.to_owned();
        let params = params.into_iter().collect::<Vec<_>>();

        let params_copy = params.clone();
        let eval = Box::new(move |table: &Table<S>, param_index: usize| {
            table.eval_constraint_for_all(&constraint, &params_copy, param_index)
        });
        DynConstraint { name, params, eval }
    }
}

/************************
 *     Solver           *
 ************************/

pub struct Solvomatic<S: State> {
    tables: Vec<Table<S>>,
    solutions: Vec<S>,
    constraints: Vec<DynConstraint<S>>,
    metadata: S::MetaData,
    config: Config,
}

impl<S: State> Solvomatic<S> {
    /// Construct an empty solver. Call `var()` and `constraint()` to give it variables and
    /// constraints, then `solve()` to solve for them.
    pub fn new(metadata: S::MetaData) -> Solvomatic<S> {
        Solvomatic {
            tables: vec![Table::new()],
            solutions: Vec::new(),
            constraints: Vec::new(),
            config: Config::default(),
            metadata,
        }
    }

    pub fn config(&mut self) -> &mut Config {
        &mut self.config
    }

    /// Add a new variable, with a set of possible values.
    pub fn var(&mut self, var: S::Var, values: impl IntoIterator<Item = S::Value>) {
        assert_eq!(self.tables.len(), 1, "Called 'var' after solving started");
        self.tables[0].add_column(var, values);
    }

    /// Add the requirement that the variables `params` must obey `constraint`.
    pub fn constraint<C: Constraint<S::Value>>(
        &mut self,
        params: impl IntoIterator<Item = S::Var>,
        constraint: C,
    ) {
        let params = params.into_iter().collect::<Vec<_>>();
        if self.config.log_constraints {
            eprintln!("Constraint {} on {:?} = {:?}", C::NAME, params, constraint);
        }

        self.constraints
            .push(DynConstraint::new(params, constraint));
    }

    fn simplify_table(&self, mut table: Table<S>) -> Option<Table<S>> {
        use YesNoMaybe::No;

        // TODO: Delete completed constraints, and log them
        // if self.config.log_completed {
        //     eprintln!"(completed constraint {} {:?}",
        //         constraint.name, constraint.params
        //     )
        // }
        /*
        let mut relevant_constraints = Vec::new();
        for constraint in &self.constraints {
            match (constraint.eval)(&table, None) {
                // Constraint is always satisfied, we can ignore it
                Yes => (),
                Maybe => relevant_constraints.push(constraint),
                No => return None,
            }
        }
        */

        loop {
            let mut to_delete: Vec<(VarIndex, EntryIndex)> = Vec::new();
            // for var in 0..table.vars.len() {
            //     if table.entries[var].len() == 1 {
            //         continue;
            //     }
            for constraint in &self.constraints {
                for param_index in 0..constraint.params.len() {
                    let answers = (constraint.eval)(&table, param_index);
                    for (entry, answer) in answers.into_iter().enumerate() {
                        if answer == No {
                            let var = &constraint.params[param_index];
                            let var_index = table.vars.iter().position(|v| v == var).unwrap();
                            let key = (var_index, entry);
                            if !to_delete.contains(&key) {
                                to_delete.push(key);
                            }
                        }
                    }
                }
            }
            //}
            if to_delete.is_empty() {
                break;
            }
            to_delete.sort();
            for (var, entry) in to_delete.iter().rev() {
                table.entries[*var].remove(*entry);
                if table.entries[*var].is_empty() {
                    return None;
                }
            }
        }
        Some(table)
    }

    fn size(&self) -> usize {
        self.tables.iter().map(|table| table.size()).sum()
    }

    fn possibilities(&self) -> f64 {
        let mut count = 0.0;
        for table in &self.tables {
            let mut product = 1.0;
            for values in &table.entries {
                product *= values.len() as f64;
            }
            count += product;
        }
        count
    }

    pub fn solve(&mut self) -> StateSet<S> {
        let start_time = Instant::now();

        while !self.tables.is_empty() {
            if self.config.log_states {
                eprintln!(
                    "{}",
                    StateSet(
                        self.tables
                            .iter()
                            .map(|table| table.into_state(&self.metadata))
                            .collect::<Vec<_>>()
                    )
                )
            }
            if self.config.log_steps {
                eprintln!(
                    "Tables = {:3}, size = {:5}, possibilities = {}",
                    self.tables.len(),
                    self.size(),
                    self.possibilities(),
                );
            }

            let table = self.tables.pop().unwrap();
            if let Some(table) = self.simplify_table(table) {
                if table.is_solved() {
                    self.solutions.push(table.into_state(&self.metadata));
                } else {
                    self.tables.extend(table.guess());
                }
            }
            if self.config.log_elapsed {
                let elapsed_time = start_time.elapsed().as_millis();
                eprintln!("  elapsed: {:5?}ms", elapsed_time);
            }
        }
        if self.config.log_steps {
            eprintln!("Total time: {}ms", start_time.elapsed().as_millis());
        }

        StateSet(mem::take(&mut self.solutions))
    }
}

/************************
 *     Config           *
 ************************/

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
