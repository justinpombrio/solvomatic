# solv-o-matic

Some puzzles require a spark of insight, a sudden recognition, or a clever twist of thought.

For all the others, there's solv-o-matic.

_Solv-o-matic is pre-alpha software. It's likely to contain bugs, and its
interface may change dramatically at any moment._

------

Solv-o-matic solves arbitrary puzzles made up of letters and numbers, that obey rules like:

- has sum 34
- is a permutation of the numbers 1..9, and
- forms word in a given word list

You declare the rules of the puzzle in a text file, and it finds all solutions for it.
Its rules are capable of expressing problems such as:

- Sudokus
- Magic Squares
- Word Squares

It is much faster than brute force, using the magical powers of distributive lattices,
though it tends to be slower than hand-rolled solvers (for the moment anyways).


**Table of contents:**

- [Example: Word Pyramid](#example-word-pyramid)
- [Documentation](#documentation)
- [Rules](#rules-1)
- [Usage](#usage)
- [Solv-o-matic as a Library](#solv-o-matic-as-a-library)

## Example: Word Pyramid

As a simple starting example, let's find all pyramids of width four that make words
when read across, and down-left, and down-right. Like this one:

```text
   /T\
  /O R\
 /S H E\
/S O M E\
```

All you need to write for this is:

```text
layout
|     /*\
|    /* *\
|   /* * *\
|  /* * * *\

range a..z
|     /*\
|    /* *\
|   /* * *\
|  /* * * *\

rule word /usr/share/dict/words
|     /.\
|    /b b\
|   /c c c\
|  /d d d d\

|     /a\
|    /b a\
|   /c b a\
|  /. c b a\

|     /a\
|    /a b\
|   /a b c\
|  /a b c .\
```

and solv-o-matic will find all 18 solutions. We'll walk through this piece by piece.

### Layout

First we need to say what the _layout_ of a "word pyramid" is. We can do that by saying
`layout`, followed by a drawing of one. Every line in the drawing needs to start with
`|` (so that solv-o-matic can tell where it starts and ends). The letters (or in other
puzzles, numbers) are marked as `*`:

```text
layout
|     /*\
|    /* *\
|   /* * *\
|  /* * * *\
```

The `/` and `\` characters are just decoration. _Every_ character that appears in a
`layout` is decoration, except for `*`s which mark where the letters (or numbers) go.

### Range

Next we need to say what the possible values of each `*` are, giving each a `range`.
In this case, they're all arbitrary letters a to z:

```text
range a..z
|     /*\
|    /* *\
|   /* * *\
|  /* * * *\
```

If we wanted to restrict the top letter to be a vowel, we could instead have said:

```text
range a e i o u
|     /*\
|    /. .\
|   /. . .\
|  /. . . .\

range a..z
|     /.\
|    /* *\
|   /* * *\
|  /* * * *\
```

### Documentation

- [Reference documentation](docs.rs/solv-o-matic)

### Rules

Most important are the rules. In this case, we're just going to write one rule,
that things need to be words:

```text
rule word /usr/share/dict/words
```

The `word` rule takes one argument, which is a path to a list of words. Given a sequence
of letters, it ensures that that sequence appears in the word list.

After the rule, we must give some layouts that say which _sequences_ the rule applies
to. The simplest way to do this is to give a single triangle for each sequence, and number
the relevant `*`s in order. For example, to say that the 2nd, 3rd, and 4th row must form
words when read from left to right, you would say:

```text
rule word /usr/share/dict/words
|     /.\
|    /1 2\
|   /. . .\
|  /. . . .\

|     /.\
|    /. .\
|   /1 2 3\
|  /. . . .\

|     /.\
|    /. .\
|   /. . .\
|  /1 2 3 4\
```

This is getting verbose, though! Fortunately there's a shorthand. We can mark each sequence
with a distinct letter like this:

```text
|     /.\
|    /b b\
|   /c c c\
|  /d d d d\
```

Solv-o-matic will turn each letter into a sequence by taking all the places that letter
appears _in order_ (top to bottom, left to right). You can use this shorthand so long as
that happens to be the correct order. If it's not, you'll need to write the numbers out.
For our example it is, so we can capture all the rule sequences with just three layouts:

```text
rule word /usr/share/dict/words
|     /.\
|    /b b\
|   /c c c\
|  /d d d d\

|     /a\
|    /b a\
|   /c b a\
|  /. c b a\

|     /a\
|    /a b\
|   /a b c\
|  /a b c .\
```

### Initial

Finally, we can fill in some known letters. In this case we probably don't want any
because there aren't many solutions. But if you really wanted the pyramid to say YAMS at
the bottom, you'd add:

```text
initial
|     /.\
|    /. .\
|   /. . .\
|  /y a m s\
```

### More Examples

For more examples, see [puzzles/](puzzles/).


## Rules

The most important part of all of this is the rules. Solv-o-matic currently supports
the following rules:

- `rule sum N`: the (positive!) numbers in the sequence sum to `n`.
- `rule product N`: the (positive!) numbers in the sequence multiply to `n`.
- `rule permutation SET`: the letters/numbers are a permutation of the given set.
- `rule subset SET`: the letters/numbers are a subsetof the given set.
- `rule superset SET`: the letters/numbers are a superset of the given set.
- `rule in_order`: the letters/numbers occur in order (each is bigger than the last).
- `rule in_reverse_order`: the letters/numbers occur in reverse order (each is smaller
  than the last).
- `rule word PATH`: the letters form a word from the word list at the given path.

Some of these rules take a "SET" as an argument. This can simply be some letters/numbers
separated by spaces (e.g. `a e i o u` for vowels). It can also include ranges using `..`
(e.g. `b..d f..h j..n p..t v..z` for consonants).


## How it Works

Solv-o-matic is based on distributive lattices, which is a fancy way to say "operations
that behave nicely with respect to 'and' and 'or'".

### State

Solv-o-matic represents the state of a partially solved puzzle as the cross product of
unions of tuples. You can think of this as a _table_ made of _partitions_. For example,
consider this table for a puzzle that has six variables A..F:

```text
    A C | B | D E F
    ----+---+------
    1 1 | 1 | 7 8 9
    1 2 | 2 |
    2 1 | 3 |
        | 4 |
```

It has three partitions `(AC, B, DEF)`. Each partition lists the possibilities for its
variables:

 - A and C are either 1,1 or 1,2 or 2,1 respectively.
 - B is between 1 and 4 inclusive.
 - D=7, E=8, and F=9

Each row in a partition is called a _tuple_ (so the first partition contains three tuples).

Overall, the table above represents _exactly_ the same state of knowledge as a table with
one partition and 12 rows:

```text
    A C B D E F
    -----------
    1 1 1 7 8 9
    1 2 1 7 8 9
    2 1 1 7 8 9
    1 1 2 7 8 9
    1 2 2 7 8 9
    2 1 2 7 8 9
    1 1 3 7 8 9
    1 2 3 7 8 9
    2 1 3 7 8 9
    1 1 4 7 8 9
    1 2 4 7 8 9
    2 1 4 7 8 9
```

But the first table is more compact. A table can potentially be _exponentially_ smaller
than it would be if you merged all of its partitions together like the above example, so
it's important we don't do that!

The _initial_ table for a puzzle has one partition per variable. For example, the starting
table for a sudoku might look like this:

```text
(1, 1) | (1, 2) | (1, 3) |
-------+--------+--------+- ...
 1     |  7     | 1      |
 2     |        | 2      |
 3     |        | 3      |
 4     |        | 4      |
 5     |        | 5      |
 6     |        | 6      |
 7     |        | 7      |
 8     |        | 8      |
 9     |        | 9      |
```

if cell (1, 2) was given to be 7.

Tables are modified in only a few ways:

- Tuples (rows in a partition) are deleted when they conflict with a rule.
- If multiple partitions only have one tuple, they're merged into a bigger partition.
- When there's nothing else to be done, solv-o-matic will try merging two partitions
  together.

Deleting tuples is what we most want to do, because it shrinks the number of possiblities.
Doing so depends on rules, so let's look at those next.

### Rules

The heart of solv-o-matic is its rules (called "constraints" in the code if you end up
looking there). Each rule needs to take a table, and output one of three possbilities:

- Yes, I am satisfied by every possibility of that table.
- No, I am never satisfied by that table.
- Maybe: I could be satisfied or dissatisfied.

How it does this depends on the rule, but there's a common pattern. It's easiest to see
with some examples.

### Rule: Sum

Let's check the rule `rule sum 18` against the sequence A B E F from the table from
before:

```text
    A C | B | D E F
    ----+---+------
    1 1 | 1 | 7 8 9
    1 2 | 2 |
    2 1 | 3 |
        | 4 |
```

I said that each rule was given "the table", but they don't need the whole table, they
only need the portion of it that's relevant to them. Thus the table is _projected_ down
to a smaller one. For the sequence A B E F, this gives:

```text
    A | B | E F
    --+---+----
    1 | 1 | 8 9
    2 | 2 |
      | 3 |
      | 4 |
```

(Notice what happened to the `A C` partition: there were three tuples but now there's
only two.)

In this tiny example, there are only 2 * 4 * 1 = 8 total possibilities represented by
this table, so we could just compute the sum for every possibility. But in general, a
table may represent exponentially many possibilities: too many to check them all.

Instead, the `sum` rule will track a _mimumum_ and _maxmimum_ possible sum, written
`[min, max]`. These min-max ranges can be combined with _and_ and with _or_.

    // You have some numbers that sum to between [a, b]
    // AND
    // some numbers that sum to between [c, d]

    [a, b] and [c, d] = [a + c, b + d]

    // You have some numbers that sum to between [a, b]
    // OR
    // some numbers that sum to between [c, d]

    [a, b] or [c, d] = [min(a, c), max(b, d)]

These two rules can be used to check if a table hasa given sum. Start by replacing
every single value with a [min, max] range:

    A     | B     | E     F
    ------+-------+------------
    [1,1] | [1,1] | [8,8] [9,9]
    [2,2] | [2,2] |
          | [3,3] |
          | [4,4] |

Then compute the min and max possible sum for each tuple by _and_ing them together
(in this case there's only one tuple):

    A     | B     | EF
    ------+-------+------------
    [1,1] | [1,1] | [17,17]
    [2,2] | [2,2] |
          | [3,3] |
          | [4,4] |

Then compute the min and max possible sum for each partition, by _or_ing them:

    A     | B     | EF
    ------+-------+------------
    [1,2] | [1,4] | [17,17]

Finally, _and_ the partitions to each other, getting the min and max total sum:

    ABEF
    ----
    [19,23]

The rule was `sum 18`, which is outside the range, so this rule answers No for this
table. It's definitely not satisfied, because the minimum possible sum of A+B+E+F is
19.

### Rule: Partition

As a more interesting rule, let's look at `rule partition 1 3 4` on A B C. Like `sum`,
`partition` is going to keep track of a min and max, but in this case it's a min and
max multiset.

_Digression: multisets._ A multiset is a set that can have multiple copies of an
element. So `{1, 1, 2}` is a multiset, and it's equal to `{1, 2, 1}` but not equal
to `{1, 2}`. Unlike sets, multisets have two different versions of "union", called
"sum" and "union". They're distinguished by how they deal with repeated elements.
The _sum_ of two multisets _adds_ the number of repetitions together, so
`{1, 1, 2} + {1, 2, 3} = {1, 1, 1, 2, 2, 3}`. The _union_ of two multisets takes the
maximum of the repetitions, so `{1, 1, 2} union {1, 2, 3} = {1, 1, 2, 3}`.

The rules for _and_ and _or_ing min/max multiset pairs are:

    [A, B] and [C, D] = [A sum C, B sum D]

    [A, B] or [C, D] = [A intersection C, B union D]

To apply this to the table, we first project the table down to just A, B, C, getting:

```text
    A C | B
    ----+--
    1 1 | 1
    1 2 | 2
    2 1 | 3
        | 4
```

Now we replace each element with a min and max multiset consisting of just itself:

```text
    A          C          | B
    ----------------------+-----------
    [{1}, {1}] [{1}, {1}] | [{1}, {1}]
    [{1}, {1}] [{2}, {2}] | [{2}, {2}]
    [{2}, {2}] [{1}, {1}] | [{3}, {3}]
                          | [{4}, {4}]
```

Then _and_ each tuple:

```text
    AC               | B
    -----------------+-----------
    [{1, 1}, {1, 1}] | [{1}, {1}]
    [{1, 2}, {1, 2}] | [{2}, {2}]
    [{1, 2}, {1, 2}] | [{3}, {3}]
                     | [{4}, {4}]
```

Then _or_ each partition:

```text
    AC               | B
    -----------------+-------------------
    [{1}, {1, 1, 2}] | [{}, {1, 2, 3, 4}]
```

Then _and_ the partitions together:

```text
    ABC
    ----------------------------
    [{1}, {1, 1, 1, 2, 2, 3, 4}]
```

Going way back to the original rule --- `permutation 1 3 4` --- we can see that it's
a superset of the minimum `{1}` and a subset of the maximum `{1, 1, 1, 2, 2, 3, 4}`,
so this rule answers Maybe.

Notice that the true answer is No! Looking at the table again, it isn't possible for
A, B, C to be a permutation of the nubmers 1, 3, 4:

```text
    A C | B
    ----+--
    1 1 | 1
    1 2 | 2
    2 1 | 3
        | 4
```

However, Maybe is also a correct answer. A rule is allowed to answer Maybe instead of
Yes or No. It's just less precise, and as a result less efficient. On that note, if you
can think of a way to check permutation cosntraints that says No here, while remaining
efficient to compute, let me know! I'll switch to it.

There are other rules, but they're all built on the same premise: answer Yes, No, or
Maybe after combining possibilities with _and_ and _or_.

### Search Strategy

So far we've walked through how a table is represented and how rules work. To put it
all together we need a strategy for using these rules to simplify tables until they're
solved. Here's a sketch of how Solv-o-matic does it:

   to simplify a table:
       for each partition P in the table:
           for each tuple T in P:
               for each rule:
                   if the rule answers No when P is set to T:
                       delete T from P

   to solve a table:
       repeat until the table has only one partition:
           merge all partitions containing only one tuple
           options = []
           for each pair of partitions P, Q:
               construct the table you'd get by merging P and Q
               simplify this table (as above)
               add this table to options
           set the table equal to the table in options with minimum size

A couple definitions used above:

- The _size_ of a table is the sum of the number of rows in each of its partitions.
- Merging two partitions means expanding out their cross product, like the earlier
 example where the table ABCDEF was expanded into 12 rows. (If both of the partitions
 have only one row, so does their cross product.)

This isn't a complete picture, for example the actual implementation does something
a bit more efficient than the three nested for loops in `to simplify a table`. But it
captures the essense of it.


## Usage

First install Rust by running the following command and following the on-screen
instructions (which will just say how to add `cargo` to your current env):

```bash
> curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Then download this repository:

```bash
> git clone git@github.com:justinpombrio/solvomatic.git
```

Finally you can run an example:

```bash
> cargo run puzzles/magic-square.txt
```

To run your own puzzle, just change the file name argument. To see options (mostly
about how much info to show while running), say `cargo run -- --help`.

## Solv-o-matic as a Library

You can also use solv-o-matic as a _library_, defining puzzles in Rust code. This is
more work but more powerful. To see how to do so, take a look at the docs (TODO:
link, for now see `src/lib.rs` or run `cargo doc --open`) and at the [examples/](examples/).
