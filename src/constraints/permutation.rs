use super::{Constraint, YesNoMaybe};
use std::fmt::Debug;
use std::hash::Hash;

/**********************
 * Constraint: Subset *
 **********************/

/// The constraint that `{X1, ..., Xn} ⊆ set`
#[derive(Debug, Clone)]
pub struct Subset<T: Debug + Hash + Eq + Ord + Clone + Sized + Send + Sync + 'static>(Bag<T>);

impl<T: Debug + Hash + Eq + Ord + Clone + Sized + Send + Sync + 'static> Subset<T> {
    pub fn new(set: impl IntoIterator<Item = T>) -> Subset<T> {
        Subset(Bag::new(set))
    }
}

impl<T: Debug + Hash + Eq + Ord + Clone + Sized + Send + Sync + 'static> Constraint<T>
    for Subset<T>
{
    type Set = BagRange<T>;

    const NAME: &'static str = "Subset";

    fn singleton(&self, _index: usize, elem: T) -> Self::Set {
        BagRange::singleton(elem)
    }

    fn and(&self, a: Self::Set, b: Self::Set) -> Self::Set {
        a.and(b)
    }

    fn or(&self, a: Self::Set, b: Self::Set) -> Self::Set {
        a.or(b)
    }

    fn check(&self, range: Self::Set) -> YesNoMaybe {
        range.is_subset(&self.0)
    }
}

/************************
 * Constraint: Superset *
 ************************/

/// The constraint that `set ⊆ {X1, ..., Xn}`
#[derive(Debug, Clone)]
pub struct Superset<T: Debug + Hash + Eq + Ord + Clone + Sized + Send + Sync + 'static>(Bag<T>);

impl<T: Debug + Hash + Eq + Ord + Clone + Sized + Send + Sync + 'static> Superset<T> {
    pub fn new(set: impl IntoIterator<Item = T>) -> Superset<T> {
        Superset(Bag::new(set))
    }
}

impl<T: Debug + Hash + Eq + Ord + Clone + Sized + Send + Sync + 'static> Constraint<T>
    for Superset<T>
{
    type Set = BagRange<T>;

    const NAME: &'static str = "Superset";

    fn singleton(&self, _index: usize, elem: T) -> Self::Set {
        BagRange::singleton(elem)
    }

    fn and(&self, a: Self::Set, b: Self::Set) -> Self::Set {
        a.and(b)
    }

    fn or(&self, a: Self::Set, b: Self::Set) -> Self::Set {
        a.or(b)
    }

    fn check(&self, range: Self::Set) -> YesNoMaybe {
        range.is_superset(&self.0)
    }
}

/*********************************
 * Constraint: SubsetAndSuperset *
 *********************************/

/// The constraint that `min ⊆ {X1, ..., Xn} ⊆ max`
#[derive(Debug, Clone)]
pub struct SubsetAndSuperset<T: Debug + Hash + Eq + Ord + Clone + Sized + Send + Sync + 'static> {
    min: Bag<T>,
    max: Bag<T>,
}

impl<T: Debug + Hash + Eq + Ord + Clone + Sized + Send + Sync + 'static> SubsetAndSuperset<T> {
    pub fn new(
        min: impl IntoIterator<Item = T>,
        max: impl IntoIterator<Item = T>,
    ) -> SubsetAndSuperset<T> {
        SubsetAndSuperset {
            min: Bag::new(min),
            max: Bag::new(max),
        }
    }
}

impl<T: Debug + Hash + Eq + Ord + Clone + Sized + Send + Sync + 'static> Constraint<T>
    for SubsetAndSuperset<T>
{
    type Set = BagRange<T>;

    const NAME: &'static str = "SubsetAndSuperset";

    fn singleton(&self, _index: usize, elem: T) -> Self::Set {
        BagRange::singleton(elem)
    }

    fn and(&self, a: Self::Set, b: Self::Set) -> Self::Set {
        a.and(b)
    }

    fn or(&self, a: Self::Set, b: Self::Set) -> Self::Set {
        a.or(b)
    }

    fn check(&self, range: Self::Set) -> YesNoMaybe {
        range.is_subset(&self.max).and(range.is_superset(&self.min))
    }
}

/***************************
 * Constraint: Permutation *
 ***************************/

/// The constraint that `{X1, ..., Xn} = expected`
#[derive(Debug, Clone)]
pub struct Permutation<T: Debug + Hash + Eq + Ord + Clone + Sized + Send + Sync + 'static> {
    expected: Bag<T>,
}

impl<T: Debug + Hash + Eq + Ord + Clone + Sized + Send + Sync + 'static> Permutation<T> {
    pub fn new(expected: impl IntoIterator<Item = T>) -> Permutation<T> {
        Permutation {
            expected: Bag::new(expected),
        }
    }
}

impl<T: Debug + Hash + Eq + Ord + Clone + Sized + Send + Sync + 'static> Constraint<T>
    for Permutation<T>
{
    type Set = BagRange<T>;

    const NAME: &'static str = "Permutation";

    fn singleton(&self, _index: usize, elem: T) -> Self::Set {
        BagRange::singleton(elem)
    }

    fn and(&self, a: Self::Set, b: Self::Set) -> Self::Set {
        a.and(b)
    }

    fn or(&self, a: Self::Set, b: Self::Set) -> Self::Set {
        a.or(b)
    }

    fn check(&self, range: Self::Set) -> YesNoMaybe {
        range.is_equal(&self.expected)
    }
}

/************************
 *     Bag Range        *
 ************************/

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BagRange<T: Ord> {
    min: Bag<T>,
    max: Bag<T>,
}

impl<T: Debug + Hash + Eq + Ord + Clone + Sized + Send + Sync + 'static> BagRange<T> {
    fn singleton(elem: T) -> BagRange<T> {
        BagRange {
            min: Bag::singleton(elem.clone()),
            max: Bag::singleton(elem),
        }
    }

    fn and(self, other: BagRange<T>) -> BagRange<T> {
        BagRange {
            min: self.min.sum(other.min),
            max: self.max.sum(other.max),
        }
    }

    fn or(self, other: BagRange<T>) -> BagRange<T> {
        BagRange {
            min: self.min.intersection(other.min),
            max: self.max.union(other.max),
        }
    }

    fn is_equal(&self, other: &Bag<T>) -> YesNoMaybe {
        use YesNoMaybe::{Maybe, No, Yes};

        if self.min.is_subset(other) && other.is_subset(&self.max) {
            if self.max.is_subset(&self.min) {
                Yes
            } else {
                Maybe
            }
        } else {
            No
        }
    }

    fn is_subset(&self, other: &Bag<T>) -> YesNoMaybe {
        use YesNoMaybe::{Maybe, No, Yes};

        if self.max.is_subset(other) {
            Yes
        } else if self.min.is_subset(other) {
            Maybe
        } else {
            No
        }
    }

    fn is_superset(&self, other: &Bag<T>) -> YesNoMaybe {
        use YesNoMaybe::{Maybe, No, Yes};

        if other.is_subset(&self.min) {
            Yes
        } else if other.is_subset(&self.max) {
            Maybe
        } else {
            No
        }
    }
}

/************************
 *     Bag              *
 ************************/

#[derive(Debug, Clone, PartialEq, Eq)]
struct Bag<T: Ord>(Vec<T>);

impl<T: Ord> Bag<T> {
    fn singleton(elem: T) -> Bag<T> {
        Bag(vec![elem])
    }

    fn new(elems: impl IntoIterator<Item = T>) -> Bag<T> {
        let mut elems = elems.into_iter().collect::<Vec<_>>();
        elems.sort();
        Bag(elems)
    }

    fn sum(self, other: Bag<T>) -> Bag<T> {
        let mut sum = Vec::new();
        let mut other_iter = other.0.into_iter().peekable();

        for x in self.0 {
            while other_iter.peek().is_some() && *other_iter.peek().unwrap() <= x {
                sum.push(other_iter.next().unwrap());
            }
            sum.push(x);
        }
        sum.extend(other_iter);

        Bag(sum)
    }

    fn union(self, other: Bag<T>) -> Bag<T> {
        let mut union = Vec::new();

        let mut iter_1 = self.0.into_iter().peekable();
        let mut iter_2 = other.0.into_iter().peekable();

        loop {
            match (iter_1.peek(), iter_2.peek()) {
                (None, None) => break,
                (Some(_), None) => {
                    let elem = iter_1.next().unwrap();
                    union.push(elem);
                }
                (None, Some(_)) => {
                    let elem = iter_2.next().unwrap();
                    union.push(elem);
                }
                (Some(x), Some(y)) if x < y => {
                    let elem = iter_1.next().unwrap();
                    union.push(elem);
                }
                (Some(x), Some(y)) if x > y => {
                    let elem = iter_2.next().unwrap();
                    union.push(elem);
                }
                (Some(_), Some(_)) => {
                    let elem = iter_1.next().unwrap();
                    iter_2.next();
                    union.push(elem);
                }
            }
        }
        Bag(union)
    }

    fn intersection(self, other: Bag<T>) -> Bag<T> {
        let mut intersection = Vec::new();
        let mut other_iter = other.0.into_iter().peekable();

        for x in self.0 {
            while other_iter.peek().is_some() && *other_iter.peek().unwrap() < x {
                other_iter.next();
            }
            if other_iter.peek().is_some() && *other_iter.peek().unwrap() == x {
                other_iter.next();
                intersection.push(x);
            }
        }

        Bag(intersection)
    }

    fn is_subset(&self, other: &Bag<T>) -> bool {
        let mut other_iter = other.0.iter().peekable();

        for x in &self.0 {
            while other_iter.peek().is_some() && **other_iter.peek().unwrap() < *x {
                other_iter.next();
            }
            let y = match other_iter.next() {
                None => return false,
                Some(y) => y,
            };
            if *y > *x {
                return false;
            }
        }
        true
    }
}

#[test]
fn test_bag() {
    fn bag(chars: &str) -> Bag<char> {
        Bag::new(chars.chars())
    }

    fn show(bag: Bag<char>) -> String {
        bag.0.into_iter().collect::<String>()
    }

    assert_eq!(show(bag("aabeeg").sum(bag("abbcf"))), "aaabbbceefg");
    assert_eq!(show(bag("aabeeg").union(bag("abbcf"))), "aabbceefg");
    assert_eq!(show(bag("abbcdff").intersection(bag("bceeffg"))), "bcff");
    assert!(bag("ace").is_subset(&bag("abccde")));
    assert!(!bag("ace").is_subset(&bag("abde")));
    assert!(bag("a").is_subset(&bag("aa")));
    assert!(bag("b").is_subset(&bag("abc")));
}
