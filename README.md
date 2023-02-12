# solv-o-matic

Some puzzles ask from you a spark of insight, or a poetic interpretation.

For all the others, there's solv-o-matic.

_Solv-o-matic is pre-alpha software. It's likely to contain bugs, and its
interface may change dramatically at any moment._

## What is it

Solv-o-matic is a constraint solver. You give it variables and constraints, then
ask it to solve the constraints for you.

It solves them pretty much the same way a dumb human would. It keeps a list of
possible values for each variable (or set of variables), and repeatedly checks
every possible value against every constraint and crosses off the ones that are
eliminated. It remembers the set of possibilities as a cross product of unions
of tuples. Like one does.

There's a small number of built in constraint types, but they're pretty
powerful. They are:

- `Sum`: some variables sum to a constant.
- `Prod`: some variables multiply to a constant. The variables must be
  non-negative!
- `Bag`: the values of some variables are a permutation of a fixed sequence. For
  example, Sudoku has the bag constraint that the first row is a permutation of
  `1..=9`.
- `Seq`: the values of some variables in order are present in a list of allowed
  sequences. For example, in the constraint
  `Seq::word_list_file("/usr/share/dict/words", 4)`, the values are letters and
  the allowed sequences are all strings of length 4 from
  `/usr/share/dict/words`.
- `Pred`: arbitrary predicate over some variables. This can express everything
  the above constraints can, but is much slower. Only use it for constraints
  that can't be expressed in other ways.

When needed, you can get a bit more power using `mapped_constraint` to modify
the value of each variable _before_ it gets passed into the constraint. For
example, if you wanted the constraint that `A + 2B + 3C = 10`, you could use
`mapped_constraint` to multiply `A` by 1 and `B` by 2 and `C` by 3 before
passing it to the constraint `Sum::new(10)`.

For more details on everything, read the docs `cargo doc --open`.

## Example

Here's a tiny example. We're going to find all two digits numbers `AB` such that
`A + B = 3`. First some imports:

```rust
use solvomatic::constraints::Sum;
use solvomatic::{Solvomatic, State};
use std::collections::HashMap;
use std::fmt;
```

First we declare a name for your puzzle state. As an empty struct. Yeah, I
know, it's kinda weird, just roll with it.

```
#[derive(Debug)]
struct Three;
```

Next we declare:

- The `Var` type (here `char`, since it will be `'A'` or `'B'`)
- The `Value` type (a `u8` will do, since it's a single digit)
- A method for displaying states. A state might not have a value for `A` or `B`;
  we'll write `_` in that case.

```
impl State for Three {
    type Var = char;
    type Value = u8;

    fn display(f: &mut String, state: &HashMap<char, u8>) -> fmt::Result {
        use std::fmt::Write;

        for letter in ['A', 'B'] {
            if let Some(digit) = state.get(&letter) {
                write!(f, "{}", digit)?;
            } else {
                write!(f, "_")?;
            }
        }
        Ok(())
    }
}
```

Finally we make a new solver, specifying the name of our state struc, tell it
the possible values of each variable (`A` and `B`), add the constraint that `A +
B = 3`, and tell it to solve! If `solve()` fails, it will produce an
`Unsatisfiable` error. Otherwise, we print the answers in `solver.table()`.

```
fn main() {
    let mut solver = Solvomatic::<Three>::new();

    solver.var('A', 1..=9);
    solver.var('B', 0..=9);

    solver.constraint(['A', 'B'], Sum::new(3));

    solver.solve().unwrap();
    println!("{}", solver.table());
}
```

And it spits out the possible solutions:

```
Step  0: size =    6 possibilities = 9                                                                                                                         
time: 0ms                                                                                                                                                      
State is one of 3:                                                                                                                                             
    30    21    12
```

There are lots more examples! They're hidden where you'd least expect them, in
`examples/`.

## Usage

[Install Rust](https://www.rust-lang.org/tools/install):

    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

Download and run an example (e.g. sudoku)

    git clone https://github.com/justinpombrio/solvomatic
    cd solvomatic
    cargo run --release --example [EXAMPLE]
    # (omit [EXAMPLE] to see the list of options)

View the docs:

    git clone https://github.com/justinpombrio/solvomatic
    cd solvomatic
    cargo doc --open

Write your own:

    // 1. make a new Rust crate
    cargo init

    // 2. in the created Cargo.toml, add:
    [dependencies]
    solvomatic = { git = "https://github.com/justinpombrio/solvomatic" }

    // 3. copy-paste an example/ from the solvomatic repo into src/main.rs

    // 4. run
    cargo run --release

    // 5. make it your own

Write your own while being a horrible software engineer:

    git clone https://github.com/justinpombrio/solvomatic
    cd solvomatic
    cp examples/palindrome.rs examples/my-thing.rs
    cargo run --release --example my-thing

**Always run with --release**, or else solvomatic will be _very_ slow.
