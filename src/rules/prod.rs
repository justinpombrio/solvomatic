use super::{Bag, BagFn};

#[derive(Debug)]
pub struct Prod;

fn add(bag_1: &Bag, bag_2: &Bag) -> Bag {
    let (min_1, max_1) = bag_1.as_range();
    let (min_2, max_2) = bag_2.as_range();
    let min = min_1 * min_2;
    let max = max_1 * max_2;
    Bag::Range { min, max }
}

impl BagFn for Prod {
    fn name(&self) -> String {
        "sum".to_owned()
    }

    fn apply(&self, args: Vec<Bag>) -> Bag {
        let mut args = args.into_iter();
        let (mut min, mut max) = args.next().expect("zero args").as_range();
        for arg in args {
            let (a_min, a_max) = arg.as_range();
            min = min * a_min;
            max = max * a_max;
        }
        Bag::Range { min, max }
    }

    /// Override for better efficiency, if you can do better than naive.
    fn parallel_apply(
        &self,
        args: Vec<Bag>,
        _parallel_index: usize,
        parallel_options: Vec<Bag>,
    ) -> Vec<Bag> {
        if args.len() == 1 {
            parallel_options
                .into_iter()
                .map(|bag| {
                    let (min, max) = bag.as_range();
                    Bag::Range { min, max }
                })
                .collect::<Vec<_>>()
        } else {
            // Multiply all but the parallel arg
            let sum = self.apply(args);
            // Now multiply each possiblity for the parallel arg
            parallel_options
                .into_iter()
                .map(|bag| add(&bag, &sum))
                .collect::<Vec<_>>()
        }
    }
}
