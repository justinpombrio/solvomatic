use super::{Constraint, YesNoMaybe};
use std::collections::{HashMap, HashSet};
use std::fmt::Debug;
use std::hash::Hash;

pub struct Count<N: Debug + Hash + Eq + Clone + Sized + 'static> {
    count_limits: HashMap<N, (u32, u32)>,
}

impl<N: Debug + Hash + Eq + Clone + Sized + 'static> Count<N> {
    pub fn new(count_limits: impl IntoIterator<Item = (N, u32, u32)>) -> Count<N> {
        let mut map = HashMap::new();
        for (var, min, max) in count_limits {
            map.insert(var, (min, max));
        }
        Count { count_limits: map }
    }
}

impl<N: Debug + Hash + Eq + Clone + Sized + 'static> Constraint<N> for Count<N> {
    type Set = HashMap<N, (u32, u32)>;

    const NAME: &'static str = "Count";

    fn singleton(&self, _index: usize, var: N) -> Self::Set {
        HashMap::from([(var, (1, 1))])
    }

    fn and(&self, a: Self::Set, b: Self::Set) -> Self::Set {
        combine_hashmaps(a, b, |(a_min, a_max), (b_min, b_max)| {
            (a_min + b_min, a_max + b_max)
        })
    }

    fn or(&self, a: Self::Set, b: Self::Set) -> Self::Set {
        combine_hashmaps(a, b, |(a_min, a_max), (b_min, b_max)| {
            (a_min.min(b_min), a_max.max(b_max))
        })
    }

    fn check(&self, set: Self::Set) -> YesNoMaybe {
        use YesNoMaybe::{Maybe, No, Yes};

        let mut satisfied = Yes;
        for (_, (actual_min, actual_max), (limit_min, limit_max)) in
            zip_hashmaps(&set, &self.count_limits)
        {
            let sat = if actual_min > limit_max || actual_max < limit_min {
                No
            } else if actual_min >= limit_min && actual_max <= limit_max {
                Yes
            } else {
                Maybe
            };
            satisfied = satisfied.and(sat);
        }
        satisfied
    }
}

fn zip_hashmaps<'a, K: Debug + Eq + Hash + Clone, V: Default + Copy>(
    map_1: &'a HashMap<K, V>,
    map_2: &'a HashMap<K, V>,
) -> impl Iterator<Item = (K, V, V)> + 'a {
    let mut keys = map_1.keys().cloned().collect::<HashSet<_>>();
    keys.extend(map_2.keys().cloned());
    keys.into_iter().map(|key| {
        (
            key.clone(),
            map_1.get(&key).copied().unwrap_or(V::default()),
            map_2.get(&key).copied().unwrap_or(V::default()),
        )
    })
}

fn combine_hashmaps<K: Debug + Eq + Hash + Clone, V: Default + Copy>(
    map_1: HashMap<K, V>,
    map_2: HashMap<K, V>,
    combine: impl Fn(V, V) -> V,
) -> HashMap<K, V> {
    zip_hashmaps(&map_1, &map_2)
        .map(|(k, v1, v2)| (k, combine(v1, v2)))
        .collect::<HashMap<_, _>>()
}
