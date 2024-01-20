//! Some puzzles ask from you a spark of insight, or a delightful recognition.
//!
//! For all the others, there's solv-o-matic.
//!
//! TODO: Overview and examples

// I'm I'll for warning against overly complex types, but Clippy is warning on some
// types that aren't very complicated.
#![allow(clippy::type_complexity)]
// No strong feelings on this but it's a reasonable way to write things and it's
// how the code currently works.
#![allow(clippy::result_unit_err)]

mod state;
mod table;

// TODO:
// - printing: show column grouping?
// - printing: log vs. stdout? Stdout vs. stderr?
// - more constraints!
// - testing!
// - command line args, including `--log` that prints after each step
// - skyscraper constraints

use constraints::Constraint;
use std::fmt;
use std::time::Instant;

pub mod constraints;

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
    apply: Box<dyn Fn(&mut Table<S>) -> Result<bool, ()>>,
    done: bool,
}

// When running `main`, this is loaded from command line args.
// See `Config` in `main.rs`.
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
        map: impl Fn(usize, S::Value) -> N + 'static,
        constraint: C,
    ) {
        let name = C::NAME.to_owned();
        let params = params.into_iter().collect::<Vec<_>>();

        if self.config.log_constraints {
            eprintln!("Constraint {} on {:?}: {:?}", name, params, constraint);
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
        while self.table.num_sections() > 1 && self.table.possibilities() > 1.0 {
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
        let start_time = Instant::now();

        // Mark completed constraints as done
        self.mark_completed_constraints();

        // Merge all constant sections together
        self.table.merge_constants();

        if self.config.log_steps {
            eprintln!(
                "\nNumber of partitions: {:2}, table size = {:4}, possibilities = {}",
                self.table.num_sections(),
                self.table.size(),
                self.table.possibilities(),
            );
        }
        if self.config.log_states {
            eprintln!("{}", self.table.display(&self.metadata));
        }

        // Consider merging all combinations of two Sections of the table
        if self.table.num_sections() > 1 {
            let mut options = Vec::new();
            for i in 0..self.table.num_sections() - 1 {
                for j in i + 1..self.table.num_sections() {
                    let mut new_table = self.table.clone();
                    new_table.merge(i, j);
                    new_table = self.apply_constraints(new_table)?;
                    options.push(new_table);
                }
            }

            // Merge the two sections that minimize the resulting table size
            let mut tables = options.into_iter();
            let mut best_table = tables.next().unwrap();
            for table in tables {
                if table.cost() < best_table.cost() {
                    best_table = table;
                }
            }
            self.table = best_table;
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
