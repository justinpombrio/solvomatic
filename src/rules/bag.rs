use super::YesNoMaybe;

// permutation, subset, superset
// sum, prod
// seq

pub type Value = i32;
pub type Var = usize;

#[derive(Debug, Clone)]
pub enum Bag {
    /// A superset of `min` but a subset of `max`. Both are sorted and can have repetition.
    /// If `min` is `[1]` and `max` is `[1, 1, 2]`: `{[1], [1, 1], [1, 2], [1, 1, 2]}`
    Multiset { min: Vec<Value>, max: Vec<Value> },
    /// Exactly one value between `min` and `max`, inclusive.
    /// `{[min], ..., [max]}`
    Range { min: Value, max: Value },
    /// Exactly one of the given set. Values are sorted and disjoint.
    /// `{[v1], ..., [vn]}`
    Set(Vec<Value>),
    /// Exactly the given value.
    /// `{[Value]}`
    Single(Value),
}

// Bad! Don't use! Loses info. (a sub b, b sub a) returns YesNoMaybes, not bools.
// Use (a sub b, b sub a) instead; 9 possibilities.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BagRel {
    // (a sub b, b sub a):
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
        use BagRel::{Incomparable, Same, Subset, Superset};
        use YesNoMaybe::{Maybe, No, Yes};

        if self.is_subset(&Bag::Set(vec![0, 1])) != Yes {
            None
        } else if self.equals(&Bag::Single(0)) == Yes {
            Some(No)
        } else if self.equals(&Bag::Single(1)) == Yes {
            Some(Yes)
        } else {
            Some(Maybe)
        }
    }

    fn equals(&self, other: &Bag) -> YesNoMaybe {
        self.is_subset(other).and(other.is_subset(self));
    }

    /*
    fn compare(&self, other: &Bag) -> BagRel {
        use Bag::{Multiset, Range, Set, Single};
        use BagRel::{Incomparable, Same, Subset, Supserset};

        match (self, other) {
            (Single(v1), Single(v2)) => {
                if v1 == v2 {
                    Same
                } else {
                    Incomparable
                }
            }
            (Single(val), Set(vals)) => {
                if vals == &[val] {
                    Same
                } else if vals.contains(val) {
                    Subset
                } else {
                    Incomparable
                }
            }
            (Set(vals), Single(val)) => {
                if vals == &[val] {
                    Same
                } else if vals.contains(val) {
                    Supserset
                } else {
                    Incomparable
                }
            }
            (Set(vals1), Set(vals2)) => match (is_subset(vals1, vals2), is_subset(vals2, vals1)) {
                (true, true) => Same,
                (true, false) => Subset,
                (false, true) => Superset,
                (false, false) => Incomparable,
            },
        }
    }
    */

    // Given set of multisets A and set of multisets B, for how many a in A, b in B
    // is it true that a is a sub-multiset of b?
    // All  -> Yes
    // Some -> Maybe
    // None -> No
    // both All and None (b.c. empty set) -> undefined
    pub fn is_subset(&self, other: &Bag) -> YesNoMaybe {
        use Bag::{Multiset, Range, Set, Single};
        use YesNoMaybe::{Maybe, No, Yes};

        match (self, other) {
            (Single(v1), Single(v2)) => YesNoMaybe::from(*v1 == *v2),
            (Single(val), Set(vals)) | (Set(vals), Single(val)) => {
                if vals == &[*val] {
                    Yes
                } else if vals.contains(val) {
                    Maybe
                } else {
                    No
                }
            }
            (Set(vals1), Set(vals2)) => {
                if vals1.len() == 1 && vals2.len() == 1 && vals1[0] == vals2[0] {
                    Yes
                } else if overlaps(vals1, vals2) {
                    Maybe
                } else {
                    No
                }
            }
            (Single(val), Range { min, max }) | (Range { min, max }, Single(val)) => {
                if *min == *val && *max == *val {
                    Yes
                } else if *min <= *val && *val <= *max {
                    Maybe
                } else {
                    No
                }
            }
            (Set(vals), Range { min, max }) | (Range { min, max }, Set(vals)) => {
                if *min == *max && vals == &[*min] {
                    Yes
                } else if vals.iter().any(|v| *min <= *v && *v <= *max) {
                    Maybe
                } else {
                    No
                }
            }
            (
                Range {
                    min: min_1,
                    max: max_1,
                },
                Range {
                    min: min_2,
                    max: max_2,
                },
            ) => {
                if min_1 == max_1 && min_2 == max_2 && min_1 == min_2 {
                    Yes
                } else if max_1 < min_2 || max_2 < min_1 {
                    No
                } else {
                    Maybe
                }
            }
            (Set(val), Multiset { min, max }) => {
                if min.contains(*val) {
                    Yes
                } else if max.contains(*val) {
                    Maybe
                } else {
                    No
                }
            }
            (Multiset { min, max }, Set(val)) => {
                if max.is_empty() || max == &[*val] {
                    Yes
                }
            }
        }
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

/// Check if `set_1` and `set_2` have any elements in common. Both are represented as a sorted
/// list.
fn overlaps(set_1: &[Value], set_2: &[Value]) -> bool {
    unimplemented!()
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
