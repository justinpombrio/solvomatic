//! From the MIT Mystery Hunt, 2023

use solvomatic::constraints::Pred;
use solvomatic::{Solvomatic, State};
use std::collections::HashMap;
use std::fmt;

#[derive(Debug)]
struct Apples;

impl State for Apples {
    type Var = &'static str;
    type Value = u32;

    fn display(f: &mut String, state: &HashMap<&'static str, u32>) -> fmt::Result {
        use std::fmt::Write;

        let mut fruits = state.keys().collect::<Vec<_>>();
        fruits.sort();
        for fruit in fruits {
            writeln!(f, "{}: {}", fruit, state.get(fruit).unwrap())?;
        }

        Ok(())
    }
}

fn is_prime(n: u32) -> bool {
    (2..n).all(|i| n % i != 0)
}

const PLUS: &[(&str, &[u32])] = &[
    ("lemon", &[3617, 4053, 4033]),
    (
        "banana",
        &[
            3287, 4229, 4230, 4231, 4232, 4233, 4234, 4235, 4236, 3311, 4678, 4757, 4011,
        ],
    ),
    ("broc", &[3082, 3277, 3320, 4547, 4060, 4548, 3160]),
    ("peach", &[3113, 3375, 4399, 4401, 4402]),
    ("eggplant", &[3089, 3090, 4081, 4601, 4602]),
    (
        "grape",
        &[
            3043, 3359, 3360, 4271, 4272, 4497, 4499, 4638, 3092, 3152, 4636, 3451,
        ],
    ),
    (
        "av",
        &[
            3080, 3354, 3509, 4046, 4221, 4222, 4223, 4224, 4225, 4226, 4227, 4228, 4770, 4771,
        ],
    ),
    ("mango", &[3363, 3364, 4051, 4311, 4312, 33653, 3042]),
    ("kiwi", &[3279, 3280, 3517, 4030, 4301]),
    ("pumpkin", &[3130, 3131, 3132, 3134, 3135, 4735]),
    (
        "orange",
        &[
            4105, 4327, 3027, 3109, 3309, 3310, 3370, 3371, 3372, 3373, 4281, 4382, 4283, 3038,
            3155, 3153,
        ],
    ),
    (
        "tomato",
        &[
            3061, 3145, 3146, 3147, 3148, 3149, 3150, 3151, 3282, 3335, 3336, 3423, 3458, 3512,
            4063, 4064, 4087, 4664, 4778, 4796, 4797, 4798, 4799, 4800, 4801, 4802, 4803, 4804,
            4805, 4806, 4807, 4808,
        ],
    ),
    ("blueberry", &[4240]),
    (
        "bean",
        &[
            3048, 3049, 4066, 4527, 4528, 4529, 4530, 4531, 4532, 4533, 4534, 4536, 4908, 4626,
        ],
    ),
];

fn main() {
    println!("Solving for fruits...");
    println!();

    let mut solver = Solvomatic::<Apples>::new();
    solver.config().log_completed = true;

    for (fruit, plus) in PLUS {
        solver.var(fruit, plus.iter().copied());
    }

    // So many PLUs for apples...
    solver.var("apple", 4000..5000);

    solver.constraint(["lemon"], Pred::new(|[a]| is_prime(*a)));
    solver.constraint(["banana", "lemon"], Pred::new(|[a, b]| is_prime(*a + *b)));
    solver.constraint(["broc", "peach"], Pred::new(|[a, b]| is_prime(*a + *b)));
    solver.constraint(["eggplant", "grape"], Pred::new(|[a, b]| is_prime(*a + *b)));
    solver.constraint(
        ["apple", "av", "mango"],
        Pred::new(|[a, b, c]| is_prime(*a + *b + *c)),
    );
    solver.constraint(
        ["kiwi", "pumpkin", "tomato"],
        Pred::new(|[a, b, c]| is_prime(*a + *b + *c)),
    );
    solver.constraint(["lemon", "peach"], Pred::new(|[a, b]| is_prime(2 * a + *b)));
    solver.constraint(
        ["lemon", "mango", "pumpkin"],
        Pred::new(|[a, b, c]| is_prime(*a + *b + *c)),
    );
    solver.constraint(
        ["mango", "pumpkin"],
        Pred::new(|[a, b]| is_prime(2 * a + *b)),
    );
    solver.constraint(
        ["apple", "kiwi", "mango"],
        Pred::new(|[a, b, c]| is_prime(*a + *b + 2 * c)),
    );
    solver.constraint(
        ["av", "blueberry", "mango", "pumpkin"],
        Pred::new(|[a, b, c, d]| is_prime(*a + *b + 2 * c + *d)),
    );
    solver.constraint(
        ["blueberry", "grape"],
        Pred::new(|[a, b]| is_prime(4 * a + *b)),
    );
    solver.constraint(
        ["kiwi", "lemon", "pumpkin", "tomato"],
        Pred::new(|[a, b, c, d]| is_prime(2 * a + *b + *c + *d)),
    );
    solver.constraint(
        ["pumpkin", "orange"],
        Pred::new(|[a, b]| is_prime(4 * a + *b)),
    );
    solver.constraint(
        ["blueberry", "grape", "peach"],
        Pred::new(|[a, b, c]| is_prime(4 * a + *b + 2 * c)),
    );
    solver.constraint(
        ["apple", "eggplant", "kiwi", "lemon", "tomato"],
        Pred::new(|[a, b, c, d, e]| is_prime(*a + 2 * b + *c + *d + 3 * e)),
    );
    solver.constraint(
        ["bean", "blueberry", "broc", "grape", "orange"],
        Pred::new(|[a, b, c, d, e]| is_prime(*a + *b + 4 * c + 2 * d + *e)),
    );
    solver.constraint(
        ["bean", "broc", "eggplant", "grape"],
        Pred::new(|[a, b, c, d]| is_prime(2 * a + *b + *c + 7 * d)),
    );

    solver.solve().unwrap();
    println!("{}", solver.table());
}
