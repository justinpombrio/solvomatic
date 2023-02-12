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
- `Permutation`: the values of some variables are a permutation of a fixed sequence. For
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

Here's an example. We'll solve a (supposedly) hard Sudoku. First some imports:

```
//! Solve a hard Sudoku

use solvomatic::constraints::{Permutation, Pred};
use solvomatic::{Solvomatic, State};
use std::fmt;
```

First we declare a name for the puzzle state. Let's use a `u8` for each digit in
the Sudoku. But actually an `Option<u8>`, because it's a rule that not all
values might be known. So we'll one a 9x9 matrix of `Option<u8>` for the whole
thing:

```
#[derive(Debug, Default)]
struct Sudoku([[Option<u8>; 9]; 9]);
```

Next we declare how `Sudoku` implements a puzzle `State`. This requires:

- The `Var` type, here `(usize, usize)` as a (row, col) index to identify a
  cell.
- The `Value` type, here a `u8` for each cell.
- A `set` function for setting the value of one cell.

```
impl State for Sudoku {
    type Var = (usize, usize);
    type Value = u8;

    fn set(&mut self, var: (usize, usize), val: u8) {
        let (i, j) = var;
        self.0[i - 1][j - 1] = Some(val);
    }
}
```

The last thing that our `State` needs is to implement `Display`, so that you can
see the board. We'll print `_` for unknown cells:

```
impl fmt::Display for Sudoku {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "+---+---+---+")?;
        for (i, row) in self.0.iter().enumerate() {
            write!(f, "|")?;
            for (j, cell) in row.iter().enumerate() {
                if let Some(n) = cell {
                    write!(f, "{:1}", n)?;
                } else {
                    write!(f, "_")?;
                }
                if j % 3 == 2 {
                    write!(f, "|")?;
                }
            }
            writeln!(f)?;
            if i % 3 == 2 {
                writeln!(f, "+---+---+---+")?;
            }
        }
        Ok(())
    }
}
```

Another requirement for `State` is that it implements `Debug` and `Default`. We
`derive`d those above.

Now let's make a Sudoku solver!

```
fn main() {
    println!("Solving a hard sudoku.");
    println!();

    let mut solver = Solvomatic::<Sudoku>::new();
```

Set the allowed values for each cell, where a cell is identified by `(i, j`):

```
    // There are 81 cells. Each cell is a number 1..9
    for i in 1..=9 {
        for j in 1..=9 {
            solver.var((i, j), 1..=9);
        }
    }
```

Set a constraint that each row, column, and 3x3 block is a permutation of the
numbers 1..9:

```
    // Each row is a permutation of 1..9
    for i in 1..=9 {
        let row: [(usize, usize); 9] = std::array::from_fn(|j| (i, j + 1));
        solver.constraint(row, Permutation::new(1..=9));
    }

    // Each col is a permutation of 1..9
    for j in 1..=9 {
        let col: [(usize, usize); 9] = std::array::from_fn(|i| (i + 1, j));
        solver.constraint(col, Permutation::new(1..=9));
    }

    // Each 3x3 block is a permutation of 1..9
    for block_i in 0..3 {
        for block_j in 0..3 {
            let mut block_cells = Vec::new();
            for i in 1..=3 {
                for j in 1..=3 {
                    block_cells.push((block_i * 3 + i, block_j * 3 + j));
                }
            }
            solver.constraint(block_cells, Permutation::new(1..=9));
        }
    }
```

Now set the constraints specific to this puzzle:

```
    // The starting config for this particular sudoku
    // (row, col, num)
    let prefilled: &[(usize, usize, u8)] = &[
        (1, 3, 5), (2, 1, 6), (1, 4, 9), (2, 5, 5),
        (2, 6, 3), (3, 4, 2), (1, 7, 4), (2, 7, 8),
        (3, 9, 3), (4, 5, 9), (5, 1, 2), (5, 8, 4),
        (6, 3, 4), (6, 5, 8), (6, 6, 5), (6, 9, 1),
        (7, 3, 2), (7, 5, 4), (7, 6, 1), (7, 9, 8),
        (8, 2, 7), (8, 7, 6), (9, 4, 3),
    ];
    for (i, j, n) in prefilled {
        solver.constraint([(*i, *j)], Pred::new(|[x]| *x == *n));
    }
```

Finally we tell it to solve! If `solve()` fails, it will produce an
`Unsatisfiable` error. Otherwise, we print the answers in `solver.table()`.

```
    solver.solve().unwrap();
    println!("{}", solver.table());
}

And it spits out the possible solutions:

```
solvomatic> cargo run --release --example sudoku
   Compiling solvomatic v0.3.0 (/home/justin/git/solvomatic)
    Finished release [optimized] target(s) in 2.32s
     Running `target/release/examples/sudoku`
Solving a hard sudoku.

Step  0: size =  139 possibilities = 160489808068608
  elapsed:   347ms
time: 362ms
State is one of 1:
    +---+---+---+
    |325|918|467|
    |649|753|812|
    |817|264|593|
    +---+---+---+
    |531|492|786|
    |286|137|945|
    |794|685|231|
    +---+---+---+
    |962|541|378|
    |173|829|654|
    |458|376|129|
    +---+---+---+
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
