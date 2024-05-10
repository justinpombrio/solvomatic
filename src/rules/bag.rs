use super::YesNoMaybe;

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
}

enum BagRel {
    // (a sub b, b sub a)
    // (true, false)
    Subset,
    // (true, true)
    Same,
    // (false, true)
    Superset,
    // (false, false)
    Incomparable,
}

impl Bag {
    pub fn as_range(&self) -> (Value, Value) {
        match self {
            Bag::Single(value) => (*value, *value),
            Bag::Set(values) => (values[0], values[values.len() - 1]),
            Bag::Range { min, max } => (*min, *max),
            Bag::Multiset { max, .. } => (max[0], max[max.len() - 1]),
        }
    }

    pub fn yes_no(&self) -> Option<YesNoMaybe> {
        use YesNoMaybe::{Maybe, No, Yes};
        use BagOrd::{Less, Equal, Greater, Incomparable};

        if self.is_subset(&Bag::Set(vec![0, 1])) != Yes {
            None
        } else if self.is_subset(&Bag::Single(0)) == Yes {
            Some(No)
        } else if self.is_subset(&Bag::Single(1)) == Yes {
            Some(Yes)
        } else {
            Some(Maybe)
        }

        /*
        // TODO: Is this exactly right? Should some of these be disallowed?
        match self {
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
        */
    }

    fn compare(&self, other: &Bag) -> BagOrd {
        use Bag::{Multiset, Range, Set, Single};
        use BagRel::{Subset, Supserset, Same, Incomparable};

        match (self, other) {
            (Single(v1), Single(v2)) =>
                if v1 == v2 {
                    Same
                } else {
                    Incomparable
                },
            (Single(val), Set(vals)) =>
                if vals == &[val] {
                    Same
                } else if vals.contains(val) {
                    Subset
                } else {
                    Incomparable
                }
            (Set(vals), Single(val)) =>
                if vals == &[val] {
                    Same
                } else if vals.contains(val) {
                    Supserset
                } else {
                    Incomparable
                }
            (Set(vals1), Set(vals2)) =>
                match (is_subset(vals1, vals2), is_subset(vals2, vals1)) {

                }
                if is_subset(vals1, vals2)
        }
    }

    pub fn is_subset(&self, other: &Bag) -> YesNoMaybe {
        use Bag::{Multiset, Range, Set, Single};
        use YesNoMaybe::{Maybe, No, Yes};

        match (self, other) {
            // single, single
            (Single(v1), Single(v2)) if v1 == v2 => Yes,
            (Single(_), Single(_)) => No,
            // single, set
            (Single(val), Set(vals)) if vals == &[*val] => Yes,
            (Single(val), Set(vals)) if vals.contains(val) => Maybe,
            (Single(_), Set(_)) => No,
            // set, single
            (Set(vals), Single(_)) if vals.is_empty() => panic!("unsat in is_subset"),
            (Set(vals), Single(val)) if vals == &[*val] => Yes,
            (Set(_), Single(_)) => No,
            // set, set
            (Set(vals1), Set(vals2)) if is_subset(vals1, vals2) => Yes,
            (Set(vals1), Set(vals2)) if is_subset(vals2, vals1) => No,
            (Set(_), Set(_)) => Maybe,
            // single, range
            (Single(val), Range { min, max }) if min == val && max == val => Yes,
            (Single(val), Range { min, max }) if min <= val && val <= max => Maybe,
            (Single(_), Range { .. }) => No,
        }
        /*
        /// A superset of `min` but a subset of `max`. Both are sorted and can have repetition.
        Multiset { min: Vec<Value>, max: Vec<Value> },
        /// Exactly one value between `min` and `max`, inclusive.
        Range { min: Value, max: Value },
        /// Exactly one of the given set. Values are sorted and disjoint.
        Set(Vec<Value>),
        /// Exactly the given value.
        Single(Value),
        */
    }

    /*
    fn equal(&self, other: &Bag) -> YesNoMaybe {
        use YesNoMaybe::{Yes, No, Maybe};

        match (self, other) {
            (Empty, _) | (_, Empty) => panic!("uncaught unsat"),
            (Single(v1), Single(v2)) => if v1 == v2 {
                Yes
            } else {
                No
            },
            (Single(val), Set(vals)) | (Set(vals), Single(val)) => if vals == &[val] {
                Yes
            } else if vals.contains(val) {
                Maybe
            } else {
                No
            }
            (Single(val), Range { min, max }) | (Range { min, max }, Single(val)) =>
                if min == val && max == val {
                    Yes
                } else if min <= val && val <= max {
                    Maybe
                } else {
                    No
                },
            (Single(val), Multiset { min, max }) | (Multiset { min, max}, Single(val)) =>
                if min == &[val] && max == &[val] {
                    Yes
                } else if min == &[val] {
                    Maybe
                } else {
                    No
                },
            (Set(vals), Range { min, max }) | (Range { min, max }, Set(vals)) =>
                ...
        }
    }

    /// A superset of `min` but a subset of `max`. Both are sorted and can have repetition.
    Multiset { min: Vec<Value>, max: Vec<Value> },
    /// Exactly one value between `min` and `max`, inclusive.
    Range { min: Value, max: Value },
    /// Exactly one of the given set. Values are sorted and disjoint.
    Set(Vec<Value>),
    */
}

/// Check if the "small" set is a subset of the "large" set. Both are represented as a sorted list.
fn is_subset(small: &[Value], big: &[Value]) -> bool {
    let mut big_iter = big.iter().peekable();

    for s in small {
        while big_iter.peek().is_some() && **big_iter.peek().unwrap() < *s {
            big_iter.next();
        }
        let b = match big_iter.next() {
            None => return false,
            Some(b) => b,
        };
        if *b > *s {
            return false;
        }
    }
    true
}
