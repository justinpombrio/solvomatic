use super::{Constraint, YesNoMaybe};
use std::fmt::Debug;
use std::marker::PhantomData;

/// The constraint that `pred(X1, ..., Xn)` holds.
pub struct Pred<T: Debug + PartialEq + Clone + Sized + 'static> {
    num_params: usize,
    pred: Box<dyn Fn(&[T]) -> bool>,
    _phantom: PhantomData<T>,
}

impl<T: Debug + PartialEq + Clone + Sized + 'static> Pred<T> {
    pub fn new<const N: usize>(pred: impl Fn(&[T; N]) -> bool + 'static) -> Pred<T> {
        let pred_generic = move |array: &[T]| -> bool {
            match TryInto::<&[T; N]>::try_into(array) {
                Ok(array) => pred(array),
                Err(msg) => panic!(
                    "Pred: wrong number of arguments in predicate function! {}",
                    msg
                ),
            }
        };
        Pred {
            num_params: N,
            pred: Box::new(pred_generic),
            _phantom: PhantomData,
        }
    }

    /// Use this instead of `new` if you don't statically know the number of params the predicate
    /// will take.
    pub fn new_with_len(len: usize, pred: impl Fn(&[T]) -> bool + 'static) -> Pred<T> {
        Pred {
            num_params: len,
            pred: Box::new(pred),
            _phantom: PhantomData,
        }
    }
}

impl<T: Debug + PartialEq + Clone + Sized + 'static> Constraint<T> for Pred<T> {
    const NAME: &'static str = "Pred";

    type Set = Vec<Option<T>>;

    fn singleton(&self, index: usize, elem: T) -> Self::Set {
        let mut result = vec![None; self.num_params];
        result[index] = Some(elem);
        result
    }

    fn and(&self, a: Self::Set, b: Self::Set) -> Self::Set {
        let mut result = a;
        for (i, elem) in b.into_iter().enumerate() {
            if let Some(elem) = elem {
                result[i] = Some(elem);
            }
        }
        result
    }

    fn or(&self, a: Self::Set, b: Self::Set) -> Self::Set {
        let mut result = a;
        for (i, elem) in b.into_iter().enumerate() {
            if &result[i] != &elem {
                result[i] = None;
            }
        }
        result
    }

    fn check(&self, set: Self::Set) -> YesNoMaybe {
        use YesNoMaybe::{Maybe, No, Yes};

        let unwrapped_set = set.into_iter().collect::<Option<Vec<T>>>();

        if let Some(set) = unwrapped_set {
            if (self.pred)(&set) {
                Yes
            } else {
                No
            }
        } else {
            Maybe
        }
    }
}

#[test]
fn test_sum() {
    use YesNoMaybe::{Maybe, No, Yes};

    let s = Pred::new(|[a, b]| a < b);

    assert_eq!(s.singleton(0, 10), [Some(10), None]);
    assert_eq!(s.singleton(1, 10), [None, Some(10)]);
    assert_eq!(s.or(s.singleton(0, 10), s.singleton(0, 20)), [None, None]);
    assert_eq!(
        s.and(s.singleton(0, 10), s.singleton(1, 20)),
        [Some(10), Some(20)]
    );

    assert_eq!(s.check(vec![None, None]), Maybe);
    assert_eq!(s.check(vec![Some(1), None]), Maybe);
    assert_eq!(s.check(vec![None, Some(1)]), Maybe);
    assert_eq!(s.check(vec![Some(1), Some(2)]), Yes);
    assert_eq!(s.check(vec![Some(2), Some(2)]), No);
}
