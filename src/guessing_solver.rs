// TODO
#![allow(unused)]

use crate::constraints::{Constraint, YesNoMaybe};
use crate::state::State;
use std::fmt;
use std::mem;

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

#[derive(Debug)]
pub struct Solution<S: State> {
    /// VarIndex -> Var
    vars: Vec<S::Var>,
    /// VarIndex -> Value
    entries: Vec<S::Value>,
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

    fn eval_constraint<C: Constraint<S::Value>>(
        &self,
        constraint: &C,
        params: &Vec<S::Var>,
        assume: Option<(VarIndex, EntryIndex)>,
    ) -> YesNoMaybe {
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

    // requires self.is_solved()!
    fn into_solution(self) -> Solution<S> {
        Solution {
            vars: self.vars,
            entries: self
                .entries
                .into_iter()
                .map(|mut vals| vals.swap_remove(0))
                .collect(),
        }
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
    name: String,
    params: Vec<S::Var>,
    eval: Box<
        dyn Fn(&Table<S>, Option<(VarIndex, EntryIndex)>) -> YesNoMaybe + Send + Sync + 'static,
    >,
}

impl<S: State> DynConstraint<S> {
    fn new<C: Constraint<S::Value>>(
        params: impl IntoIterator<Item = S::Var>,
        constraint: C,
    ) -> DynConstraint<S> {
        let name = C::NAME.to_owned();
        let params = params.into_iter().collect::<Vec<_>>();

        let params_copy = params.clone();
        let eval = Box::new(
            move |table: &Table<S>, assume: Option<(VarIndex, EntryIndex)>| {
                table.eval_constraint(&constraint, &params_copy, assume)
            },
        );
        DynConstraint { name, params, eval }
    }
}

/************************
 *     Solver           *
 ************************/

pub struct GuessingSolver<S: State> {
    tables: Vec<Table<S>>,
    solutions: Vec<Solution<S>>,
    constraints: Vec<DynConstraint<S>>,
    metadata: S::MetaData,
    config: Config,
}

impl<S: State> GuessingSolver<S> {
    /// Construct an empty solver. Call `var()` and `constraint()` to give it variables and
    /// constraints, then `solve()` to solve for them.
    pub fn new(metadata: S::MetaData) -> GuessingSolver<S> {
        GuessingSolver {
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
        self.constraints
            .push(DynConstraint::new(params, constraint));
    }

    fn delete_completed_constraints(&mut self) {
        // TODO
    }

    // returns true if the table "simplified" away to nothing
    fn simplify_table(&mut self, table_index: usize) -> bool {
        use YesNoMaybe::{Maybe, No, Yes};

        loop {
            let mut to_delete: Vec<(VarIndex, EntryIndex)> = Vec::new();
            for var in 0..self.tables[table_index].vars.len() {
                for entry in 0..self.tables[table_index].entries[var].len() {
                    for constraint in &self.constraints {
                        match (constraint.eval)(&self.tables[table_index], Some((var, entry))) {
                            Yes | Maybe => (),
                            No => {
                                to_delete.push((var, entry));
                                break;
                            }
                        }
                    }
                }
            }
            if to_delete.is_empty() {
                break;
            }
            for (var, entry) in to_delete.iter().rev() {
                self.tables[table_index].entries[*var].remove(*entry);
                if self.tables[table_index].entries[*var].is_empty() {
                    return true;
                }
            }
        }
        false
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

    fn simplify(&mut self) {
        println!(
            "Simplify. Num tables: {}, num possibilities: {}",
            self.tables.len(),
            self.possibilities()
        );
        //for table in &self.tables {
        //    println!("{}", table);
        //}

        for i in (0..self.tables.len()).rev() {
            if self.simplify_table(i) {
                self.tables.remove(i); // unsat
            }
        }
        for i in (0..self.tables.len()).rev() {
            if self.tables[i].is_solved() {
                self.solutions.push(self.tables.remove(i).into_solution());
            }
        }
    }

    pub fn solve(&mut self) -> Vec<Solution<S>> {
        self.simplify();
        while let Some(table) = self.tables.pop() {
            self.tables.extend(table.guess());
            self.simplify();
        }
        mem::take(&mut self.solutions)
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
