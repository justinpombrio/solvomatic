use crate::constraints::{Constraint, YesNoMaybe};
use crate::state::State;
use std::fmt;

pub type VarIndex = usize;
pub type EntryIndex = usize;

pub struct Table<S: State> {
    /// VarIndex -> Var
    pub vars: Vec<S::Var>,
    /// VarIndex -> set of Value
    pub entries: Vec<Vec<S::Value>>,
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
    pub fn new() -> Table<S> {
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

    pub fn size(&self) -> usize {
        let mut size = 0;
        for values in &self.entries {
            size += values.len();
        }
        size
    }

    pub fn possibilities(&self) -> f64 {
        let mut product = 1.0;
        for values in &self.entries {
            product *= values.len() as f64;
        }
        product
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

    pub fn guess(self) -> Vec<Table<S>> {
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

    pub fn eval_constraint_for_all<C: Constraint<S::Value>>(
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

    pub fn is_solved(&self) -> bool {
        for values in &self.entries {
            if values.len() > 1 {
                return false;
            }
        }
        true
    }

    pub fn into_state(&self, metadata: &S::MetaData) -> S {
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
