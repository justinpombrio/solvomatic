use crate::state::State;
use crate::table::Table;
use std::fmt::Debug;

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
            (Maybe, Maybe) | (Yes, Maybe) | (Maybe, Yes) => Maybe,
            (No, _) | (_, No) => No,
        }
    }
}

pub type Value = i32;
pub type Var = usize;

#[derive(Debug, Clone)]
pub enum Bag {
    /// A superset of `min` but a subset of `max`. Both are sorted and can have repetition.
    Multiset { min: Vec<Value>, max: Vec<Value> },
    /// Exactly one value between `min` and `max`, inclusive.
    Range { min: Value, max: Value },
    /// Exactly one of the given set. Values are sorted and disjoint.
    Set(Vec<Value>),
    /// Exactly the given value.
    Single(Value),
    /// Unsatisfiable!
    Empty,
}

impl Bag {
    fn as_range(&self) -> Option<(Value, Value)> {
        match self {
            Bag::Empty => None,
            Bag::Single(value) => Some((*value, *value)),
            Bag::Set(values) => Some((values[0], values[values.len() - 1])),
            Bag::Range { min, max } => Some((*min, *max)),
            Bag::Multiset { max, .. } => Some((max[0], max[max.len() - 1])),
        }
    }

    fn yes_no(&self) -> Option<YesNoMaybe> {
        use YesNoMaybe::{Maybe, No, Yes};

        // TODO: Is this exactly right? Should some of these be disallowed?
        match self {
            Bag::Empty => None,
            Bag::Single(0) => Some(No),
            Bag::Single(1) => Some(Yes),
            Bag::Single(_) => None,
            Bag::Set(vals) => match vals.as_slice() {
                &[0] => Some(No),
                &[1] => Some(Yes),
                &[0, 1] => Some(Maybe),
                _ => None,
            },
            Bag::Range { min, max } => match (min, max) {
                (0, 0) => Some(No),
                (1, 1) => Some(Yes),
                (0, 1) => Some(Maybe),
                (_, _) => None,
            },
            Bag::Multiset { min, max } => match (min.as_slice(), max.as_slice()) {
                (&[0], &[0]) => Some(No),
                (&[1], &[1]) => Some(Yes),
                (&[], &[0, 1]) => Some(Maybe),
                (_, _) => None,
            },
        }
    }
}

pub trait Rule<S: State>: Debug + Send + Sync + 'static {
    /// A name for this rule, for debugging purposes.
    fn name(&self) -> String;

    fn vars(&self) -> &[S::Var];

    fn check(&self, table: &Table<S>) -> YesNoMaybe;

    fn parallel_check(&self, table: &Table<S>, parallel_var: Var) -> Vec<YesNoMaybe>;
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
enum Formula {
    Var([Var; 1]),
    BagFn {
        vars: Vec<Var>,
        bag_fn: Box<dyn BagFn>,
        formulae: Vec<Formula>,
    },
}

impl Formula {
    fn vars(&self) -> &[Var] {
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
                        args.push(Bag::Empty); // never seen
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

impl<S> Rule<S> for Formula
where
    S: State<Value = Value, Var = Var>,
{
    fn name(&self) -> String {
        match self {
            Formula::Var([var]) => format!("${}", var),
            Formula::BagFn { bag_fn, .. } => format!("{}", bag_fn.name()),
        }
    }

    fn vars(&self) -> &[Var] {
        Formula::vars(self)
    }

    fn check(&self, table: &Table<S>) -> YesNoMaybe {
        self.eval(table).yes_no().unwrap_or_else(|| {
            panic!(
                "Rule {} did not produce a boolean result",
                Rule::<S>::name(self)
            )
        })
    }

    fn parallel_check(&self, table: &Table<S>, parallel_var: S::Var) -> Vec<YesNoMaybe> {
        self.parallel_eval(table, parallel_var)
            .into_iter()
            .map(|bag| {
                bag.yes_no().unwrap_or_else(|| {
                    panic!(
                        "Rule {} did not produce a boolean result",
                        Rule::<S>::name(self)
                    )
                })
            })
            .collect::<Vec<_>>()
    }
}
