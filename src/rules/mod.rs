use crate::state::State;
use crate::table::Table;
use std::fmt::Debug;
use std::from::From;

mod bag;
mod prod;
mod sum;

pub use bag::{Bag, Value, Var};
pub use prod::Prod;
pub use sum::Sum;

// TODO: Docs

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum YesNoMaybe {
    Yes,
    No,
    Maybe,
}

impl YesNoMaybe {
    pub fn and(&self, other: YesNoMaybe) -> YesNoMaybe {
        use YesNoMaybe::{Maybe, No, Yes};

        match (self, other) {
            (Yes, Yes) => Yes,
            (No, _) | (_, No) => No,
            (Maybe, _) | (_, Maybe) => Maybe,
        }
    }
}

impl From<bool> for YesNoMaybe {
    fn from(b: bool) -> YesNoMaybe {
        if b {
            Yes
        } else {
            No
        }
    }
}

pub trait BagFn: Debug + Send + Sync + 'static {
    fn name(&self) -> String;

    fn apply(&self, args: Vec<Bag>) -> Bag;

    /// Override for better efficiency, if you can do better than naive.
    fn parallel_apply(
        &self,
        args: Vec<Bag>,
        parallel_index: usize,
        parallel_options: Vec<Bag>,
    ) -> Vec<Bag> {
        let mut output = Vec::new();
        for value in parallel_options {
            let bags = args
                .iter()
                .enumerate()
                .map(|(i, bag)| {
                    if i == parallel_index {
                        value.clone()
                    } else {
                        bag.clone()
                    }
                })
                .collect::<Vec<_>>();
            output.push(self.apply(bags));
        }
        output
    }
}

#[derive(Debug)]
pub enum Formula {
    Var([Var; 1]),
    BagFn {
        vars: Vec<Var>,
        bag_fn: Box<dyn BagFn>,
        formulae: Vec<Formula>,
    },
}

impl Formula {
    pub fn name(&self) -> String {
        match self {
            Formula::Var([var]) => format!("${}", var),
            Formula::BagFn { bag_fn, .. } => format!("{}", bag_fn.name()),
        }
    }

    pub fn vars(&self) -> &[Var] {
        match self {
            Formula::Var(var_slice) => var_slice,
            Formula::BagFn { vars, .. } => vars,
        }
    }

    fn eval<S>(&self, table: &Table<S>) -> Bag
    where
        S: State<Value = Value, Var = Var>,
    {
        match self {
            Formula::Var([var]) => Bag::Set(table.entries[table.var_index(var)].clone()),
            Formula::BagFn {
                bag_fn, formulae, ..
            } => bag_fn.apply(
                formulae
                    .into_iter()
                    .map(|f| f.eval(table))
                    .collect::<Vec<_>>(),
            ),
        }
    }

    pub fn check<S>(&self, table: &Table<S>) -> YesNoMaybe
    where
        S: State<Value = Value, Var = Var>,
    {
        self.eval(table)
            .yes_no()
            .unwrap_or_else(|| panic!("Rule {} did not produce a boolean result", self.name()))
    }

    pub fn parallel_check<S>(&self, table: &Table<S>, parallel_var: Var) -> Vec<YesNoMaybe>
    where
        S: State<Value = Value, Var = Var>,
    {
        self.parallel_eval(table, parallel_var)
            .into_iter()
            .map(|bag| {
                bag.yes_no().unwrap_or_else(|| {
                    panic!("Rule {} did not produce a boolean result", self.name())
                })
            })
            .collect::<Vec<_>>()
    }

    fn parallel_eval<S>(&self, table: &Table<S>, parallel_var: Var) -> Vec<Bag>
    where
        S: State<Value = Value, Var = Var>,
    {
        match self {
            Formula::Var([var]) => {
                assert_eq!(*var, parallel_var);
                let values = &table.entries[table.var_index(var)];
                values
                    .iter()
                    .map(|val| Bag::Single(*val))
                    .collect::<Vec<_>>()
            }
            Formula::BagFn {
                bag_fn, formulae, ..
            } => {
                let mut args = Vec::new();
                let mut parallel_index = 0;
                let mut parallel_options = Vec::new();
                for (i, formula) in formulae.iter().enumerate() {
                    if formula.vars().contains(&parallel_var) {
                        parallel_options = formula.parallel_eval(table, parallel_var);
                        parallel_index = i;
                    } else {
                        args.push(formula.eval(table));
                    }
                }
                bag_fn.parallel_apply(args, parallel_index, parallel_options)
            }
        }
    }
}
