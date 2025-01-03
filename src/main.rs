//! See README.md

use argh::FromArgs;
use parser_ll1::{CompiledParser, Grammar, GrammarError, Parser};
use solvomatic::{Solvomatic, State};
use std::collections::HashMap;
use std::fmt;
use std::fs;
use std::iter::Peekable;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Arc;

const STAR: i32 = -27; // entry representing '*'

/************************
 *     Entry            *
 ************************/

/// One of three things:
///
/// - A positive number
/// - A letter, represented as a negative number (a=-1, b=-2, etc.)
/// - '.', meaning unknown, represented as None
#[derive(Debug, Clone, Copy)]
struct Entry(Option<i32>);

fn read_letter(letter: char) -> Option<i32> {
    if letter.is_ascii_uppercase() {
        // upper case letter -> negative one-based number index
        Some('A' as i32 - letter as i32 - 1)
    } else if letter.is_ascii_lowercase() {
        // lower case letter -> negative one-based number index
        Some('a' as i32 - letter as i32 - 1)
    } else {
        None
    }
}

impl Entry {
    fn parse(input: &mut Peekable<impl Iterator<Item = char>>) -> Option<Entry> {
        let ch = match input.next() {
            Some(ch) => ch,
            None => return None,
        };
        if ch == '.' {
            Some(Entry(None))
        } else if ch == '*' {
            // '*' shouldn't occur in actual input, only templates
            Some(Entry(Some(STAR)))
        } else if ch.is_ascii_alphabetic() {
            let entry = read_letter(ch)?;
            Some(Entry(Some(entry)))
        } else if let Some(digit) = ch.to_digit(10) {
            let mut n = digit as i32;
            while let Some(ch) = input.peek() {
                if let Some(digit) = ch.to_digit(10) {
                    n *= 10;
                    n += digit as i32;
                    input.next();
                } else {
                    break;
                }
            }
            Some(Entry(Some(n)))
        } else {
            None
        }
    }
}

impl fmt::Display for Entry {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.0 {
            Some(n) if n < 0 => write!(f, "{}", (-n + 64) as u8 as char),
            Some(n) => write!(f, "{}", n),
            None => write!(f, "."),
        }
    }
}

/************************
 *     Layout           *
 ************************/

/// A layout saying how to read and write one `Data` state.
#[derive(Debug, Clone, Default)]
struct Layout {
    layout: String,
    num_entries: usize,
}

impl Layout {
    fn new(input: &str) -> Layout {
        Layout {
            layout: input.to_owned(),
            num_entries: input.chars().filter(|ch| *ch == '*').count(),
        }
    }

    fn parse_full_input(&self, input: &str) -> Result<Vec<Entry>, BadInput> {
        fn get_line_for_error_message(text: &str, remaining_chars: usize) -> String {
            let char_index = text.chars().count().saturating_sub(remaining_chars + 1);
            let line_index = text
                .chars()
                .take(char_index)
                .filter(|ch| *ch == '\n')
                .count();
            text.lines().nth(line_index).unwrap_or("[empty]").to_owned()
        }

        let mut layout_iter = self.layout.chars().peekable();
        let mut input_iter = input.chars().peekable();
        match parse_layout(&mut layout_iter, &mut input_iter, 0) {
            Some(entries) => Ok(entries),
            None => Err(BadInput::DoesNotMatchLayout(
                get_line_for_error_message(&self.layout, layout_iter.count()),
                get_line_for_error_message(input, input_iter.count()),
            )),
        }
    }

    fn parse_sub_input(&self, input: &str) -> Result<Vec<Vec<Entry>>, BadInput> {
        let mut results = Vec::new();
        for offset in 0..self.layout.len() {
            let mut layout_iter = self.layout.chars().peekable();
            let mut input_iter = input.chars().peekable();
            if let Some(entries) = parse_layout(&mut layout_iter, &mut input_iter, offset) {
                results.push(entries);
            }
        }
        if results.is_empty() {
            Err(BadInput::NoMatches(input.to_owned()))
        } else {
            Ok(results)
        }
    }
}

#[allow(clippy::while_let_on_iterator)] // Clippy's wrong, for loop would take ownership
fn parse_layout(
    layout: &mut Peekable<impl Iterator<Item = char>>,
    input: &mut Peekable<impl Iterator<Item = char>>,
    initial_offset: usize,
) -> Option<Vec<Entry>> {
    let mut entries = Vec::new();

    // 1. Skip past `initial_offset` characters, and record the final indentation (column).
    let indent = {
        let mut ind = 0;
        for _ in 0..initial_offset {
            if let Some(ch) = layout.next() {
                if ch == '\n' {
                    ind = 0;
                } else if ch == '*' {
                    entries.push(Entry(None));
                    ind += 1;
                } else {
                    ind += 1;
                }
            } else {
                panic!("Bad call to parse_layout: index OOB");
            }
        }
        ind
    };

    // 2. Match the input against the layout.
    while let Some(input_ch) = input.peek().copied() {
        if input_ch == '\n' {
            // Consume until next newline, in both layout and input
            input.next();
            while let Some(ch) = layout.next() {
                if ch == '*' {
                    entries.push(Entry(None));
                } else if ch == '\n' {
                    break;
                }
            }
            // Skip indentation in the layout
            for _ in 0..indent {
                if layout.peek().copied() == Some('\n') {
                    break;
                }
                let ch = layout.next();
                if ch == Some('*') {
                    entries.push(Entry(None));
                }
            }
            continue;
        }

        match layout.next() {
            Some('*') => {
                let entry = Entry::parse(input)?;
                entries.push(entry);
            }
            Some(ch) => {
                if input.next() != Some(ch) {
                    return None;
                }
            }
            None => {
                if input.next().is_none() {
                    return Some(entries);
                } else {
                    return None;
                }
            }
        }
    }

    // 3. Add any remaining entries.
    for ch in layout {
        if ch == '*' {
            entries.push(Entry(None));
        }
    }

    Some(entries)
}

/************************
 *     Data             *
 ************************/

#[derive(Debug, Default)]
struct Data {
    entries: Vec<Entry>,
    layout: Arc<Layout>,
}

impl State for Data {
    type Var = usize;
    type Value = i32;
    type MetaData = Arc<Layout>;

    fn set(&mut self, var: usize, val: i32) {
        self.entries[var] = Entry(Some(val));
    }

    fn new(layout: &Arc<Layout>) -> Data {
        Data {
            entries: vec![Entry(None); layout.num_entries],
            layout: layout.clone(),
        }
    }
}

impl Data {
    fn new(input: &str, layout: Arc<Layout>) -> Result<Data, BadInput> {
        Ok(Data {
            entries: layout.parse_full_input(input)?,
            layout,
        })
    }
}

impl fmt::Display for Data {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let entries = self
            .entries
            .iter()
            .map(|e| e.to_string())
            .collect::<Vec<_>>();
        let max_len = entries.iter().map(|s| s.len()).max().unwrap_or(1);

        let mut whitespace = self.layout.layout.split('*');
        write!(f, "{}", whitespace.next().unwrap())?;
        for (i, entry) in self.entries.iter().enumerate() {
            write!(
                f,
                "{:>padding$}{}{}",
                "",
                entry,
                whitespace.next().unwrap(),
                padding = max_len - entries[i].len(),
            )?;
        }
        Ok(())
    }
}

/************************
 *     Input Errors     *
 ************************/

#[derive(Debug)]
enum BadInput {
    BadRangeEntry(i32),
    DoesNotMatchLayout(String, String),
    NoMatches(String),
}

impl fmt::Display for BadInput {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            BadInput::DoesNotMatchLayout(input, layout) => {
                write!(f, "Input '{}' does not match layout '{}'.", input, layout)
            }
            BadInput::BadRangeEntry(entry) => write!(f, "Bad range entry {}", entry),
            BadInput::NoMatches(input) => {
                write!(f, "Pattern does not match the layout:\n{}", input)
            }
        }
    }
}

/************************
 *     Puzzle           *
 ************************/

#[derive(Debug, Clone)]
struct PuzzleRange {
    possibilities: Vec<i32>,
    data: String,
}

#[derive(Debug, Clone)]
enum PuzzleRule {
    Sum(i32),
    Prod(i32),
    Word(String),
    Permutation(Vec<i32>),
    Superset(Vec<i32>),
    Subset(Vec<i32>),
    InOrder(bool),
}

#[derive(Debug, Clone)]
struct PuzzleRuleSet {
    rules: Vec<PuzzleRule>,
    datas: Vec<String>,
}

#[derive(Debug)]
struct PuzzleDefinition {
    layout: String,
    ranges: Vec<PuzzleRange>,
    rule_sets: Vec<PuzzleRuleSet>,
    initial: Option<String>,
}

struct WordListLoader {
    cache: HashMap<PathBuf, String>,
}

impl WordListLoader {
    fn new() -> WordListLoader {
        WordListLoader {
            cache: HashMap::new(),
        }
    }

    fn load(&mut self, path: &str, word_len: usize) -> solvomatic::constraints::Seq<i32> {
        let word_list = self
            .cache
            .entry(PathBuf::from(path))
            .or_insert_with(|| fs::read_to_string(path).expect("Failed to load word list"));

        let words = word_list
            .lines()
            .map(|s| s.trim())
            .map(|s| s.to_lowercase())
            .filter(|s| s.chars().count() == word_len)
            .map(|s| s.chars().map(|ch| 96 - (ch as i32)).collect::<Vec<_>>());

        solvomatic::constraints::Seq::new(word_len, words)
    }
}

impl PuzzleDefinition {
    fn make_solver(self, config: Config) -> Result<Solvomatic<Data>, BadInput> {
        use solvomatic::constraints::{Permutation, Pred, Prod, Subset, Sum, Superset};

        let mut word_list_loader = WordListLoader::new();

        let original_layout = Layout::new(&self.layout);
        let layout = Arc::new(original_layout.clone());
        let mut solver = Solvomatic::new(layout.clone());

        if !config.quiet {
            solver.config().log_steps = true;
            solver.config().log_constraints = config.log_constraints;
            solver.config().log_completed = config.log_completed;
            solver.config().log_elapsed = config.log_elapsed;
            solver.config().log_states = config.log_states;
        }

        for range in &self.ranges {
            let data = Data::new(&range.data, layout.clone())?;
            for (i, entry) in data.entries.iter().enumerate() {
                match entry {
                    Entry(None) => (),
                    Entry(Some(STAR)) => solver.var(i, range.possibilities.iter().copied()),
                    Entry(Some(n)) => return Err(BadInput::BadRangeEntry(*n)),
                }
            }
        }

        for rule_set in &self.rule_sets {
            let mut var_lists = Vec::new();
            for data in &rule_set.datas {
                for entries in layout.parse_sub_input(data)? {
                    let mut key_to_var_list = HashMap::new();
                    for (i, entry) in entries.iter().enumerate() {
                        if let Entry(Some(entry)) = entry {
                            let key = if *entry >= 0 {
                                // entry was a number
                                None
                            } else {
                                // entry was a letter, or '*'
                                Some(entry)
                            };
                            key_to_var_list
                                .entry(key)
                                .or_insert_with(Vec::new)
                                .push((i, entry))
                        }
                    }
                    for (key, mut var_list) in key_to_var_list {
                        if key.is_none() {
                            var_list.sort_by_key(|(_, entry)| *entry);
                        }
                        let var_list = var_list.into_iter().map(|(var, _)| var).collect::<Vec<_>>();
                        var_lists.push(var_list)
                    }
                }
            }
            for var_list in var_lists {
                for rule in &rule_set.rules {
                    let vars = var_list.iter().copied();
                    match rule {
                        PuzzleRule::Sum(sum) => solver.constraint(vars, Sum::new(*sum)),
                        PuzzleRule::Prod(prod) => solver.constraint(vars, Prod::new(*prod)),
                        PuzzleRule::Word(path) => {
                            let words = word_list_loader.load(path, var_list.len());
                            solver.constraint(vars, words);
                        }
                        PuzzleRule::Permutation(permutation) => {
                            solver.constraint(vars, Permutation::new(permutation.iter().copied()))
                        }
                        PuzzleRule::Subset(set) => {
                            solver.constraint(vars, Subset::new(set.iter().copied()))
                        }
                        PuzzleRule::Superset(set) => {
                            solver.constraint(vars, Superset::new(set.iter().copied()))
                        }
                        PuzzleRule::InOrder(ascending) => {
                            let len = var_list.len();
                            solver.constraint(
                                vars,
                                if *ascending {
                                    Pred::with_len(len, |elems: &[i32]| {
                                        // Convert letters to numbers
                                        elems.windows(2).all(|w| w[0].abs() <= w[1].abs())
                                    })
                                } else {
                                    Pred::with_len(len, |elems: &[i32]| {
                                        // Convert letters to numbers
                                        elems.windows(2).all(|w| w[1].abs() <= w[0].abs())
                                    })
                                },
                            )
                        }
                    }
                }
            }
        }

        if let Some(initial) = self.initial {
            let data = Data::new(&initial, layout.clone())?;
            for (i, entry) in data.entries.iter().enumerate() {
                match entry {
                    Entry(None) => (),
                    Entry(Some(n)) => {
                        let n = *n;
                        solver.constraint([i], Pred::new(move |[x]| *x == n))
                    }
                }
            }
        }

        Ok(solver)
    }
}

/************************
 *     Parser           *
 ************************/

fn make_puzzle_parser() -> Result<impl CompiledParser<PuzzleDefinition>, GrammarError> {
    use parser_ll1::{choice, tuple};

    // Whitespace includes '#' comments that extend to the end of the line
    let mut g = Grammar::with_whitespace("([ \t\n]+|#[^\n]*\n)+")?;

    // A data section is a sequence of lines that all start with '|'
    let data_p = g.regex("template", "(\\|[^\n]*\n)+")?.span(|span| {
        let mut stripped = String::new();
        for line in span.substr.lines() {
            stripped.push_str(&line[1..]);
            stripped.push('\n');
        }
        stripped
    });

    // An entry (i32) is either a letter or a numeral
    let letter_p = g
        .regex("letter", "[a-zA-Z]")?
        .span(|span| read_letter(span.substr.chars().next().unwrap()).unwrap());
    let numeral_p = g
        .regex("numeral", "[0-9]+")?
        .try_span(|span| i32::from_str(span.substr));
    let entry_p = choice("letter or numeral", (letter_p, numeral_p));

    // An entry set is a set of entries and `entry .. entry` ranges.
    let entry_range_p = tuple(
        "letter/numeral range",
        (
            entry_p.clone(),
            entry_p.clone().preceded(g.string("..")?).opt(),
        ),
    )
    .map(|(a, opt_b)| {
        if let Some(b) = opt_b {
            let min = a.min(b);
            let max = a.max(b);
            (min..=max).collect::<Vec<i32>>()
        } else {
            vec![a]
        }
    });
    let entry_set_p = entry_range_p
        .clone()
        .fold_many1(entry_range_p, |mut vec1, vec2| {
            vec1.extend(vec2);
            vec1
        });

    // layout
    //   DATA
    let layout_p = tuple("layout", (g.string("layout")?, data_p.clone())).map(|(_, data)| data);

    // range arg...
    let range_p = entry_set_p.clone().preceded(g.string("range")?);
    // range arg...
    //   DATA
    //   ...
    let range_and_data_p =
        tuple("range", (range_p, data_p.clone())).map(|(possibilities, data)| PuzzleRange {
            possibilities,
            data,
        });

    // rule name arg...
    //   DATA
    //   ...
    let path = g
        .regex("path", "([_/a-zA-Z0-9-]|\\.[_a-zA-Z])+")?
        .span(|span| span.substr.to_owned());
    let sum_p = entry_p
        .clone()
        .preceded(g.string("sum")?)
        .map(PuzzleRule::Sum);
    let prod_p = entry_p
        .clone()
        .preceded(g.string("prod")?)
        .map(PuzzleRule::Prod);
    let permutation_p = tuple(
        "permutation rule",
        (g.string("permutation")?, entry_set_p.clone()),
    )
    .map(|(_, entries)| PuzzleRule::Permutation(entries));
    let subset_p = tuple("subset rule", (g.string("subset")?, entry_set_p.clone()))
        .map(|(_, entries)| PuzzleRule::Subset(entries));
    let superset_p = tuple(
        "superset rule",
        (g.string("superset")?, entry_set_p.clone()),
    )
    .map(|(_, entries)| PuzzleRule::Superset(entries));
    let word_p = path.preceded(g.string("word")?).map(PuzzleRule::Word);
    let in_order_p = g.string("in_order")?.constant(PuzzleRule::InOrder(true));
    let in_reverse_order_p = g
        .string("in_reverse_order")?
        .constant(PuzzleRule::InOrder(false));

    let rule_p = choice(
        "rule name ('sum', 'prod', 'word', 'permutation', 'subset', 'supserset', 'in_order')",
        (
            sum_p,
            prod_p,
            word_p,
            permutation_p,
            subset_p,
            superset_p,
            in_order_p,
            in_reverse_order_p,
        ),
    );
    let rules_p = rule_p.preceded(g.string("rule")?).many1();
    let rule_set_p = tuple("rules", (rules_p, data_p.clone().many1()))
        .map(|(rules, datas)| PuzzleRuleSet { rules, datas });

    // initial
    //   DATA
    let initial_p = tuple("initial", (g.string("initial")?, data_p)).map(|(_, data)| data);

    // layout
    //   DATA
    // range min max
    //   DATA
    //   ...
    // ...
    // rule name arg ...
    //   DATA
    //   ...
    // ...
    // initial
    //   DATA
    // ?
    let puzzle_p = tuple(
        "Puzzle definition",
        (
            layout_p,
            range_and_data_p.many1(),
            rule_set_p.many1(),
            initial_p.opt(),
        ),
    )
    .map(|(layout, ranges, rule_sets, initial)| PuzzleDefinition {
        layout,
        ranges,
        rule_sets,
        initial,
    });

    g.compile_parser(puzzle_p)
}

/************************
 *     Main             *
 ************************/

/// solv-o-matic
#[derive(Debug, Clone, FromArgs)]
struct Config {
    /// the puzzle definition file to run
    #[argh(positional)]
    filename: String,

    /// don't log anything besides the solution
    #[argh(switch, short = 'q', long = "quiet")]
    quiet: bool,

    /// log the list of contraints before solving
    #[argh(switch, long = "log-constraints")]
    log_constraints: bool,

    /// log when a constraint is completed
    #[argh(switch, long = "log-complete")]
    log_completed: bool,

    /// log how long each step took
    #[argh(switch, long = "log-elapsed")]
    log_elapsed: bool,

    /// log intermediate states (these can be very large!)
    #[argh(switch, long = "log-states")]
    log_states: bool,
}

fn main() {
    let config = argh::from_env::<Config>();

    let parser = make_puzzle_parser().unwrap_or_else(|err| panic!("{}", err));
    let file_contents = fs::read_to_string(&config.filename).unwrap();
    let puzzle_definition = parser
        .parse(&config.filename, &file_contents)
        .unwrap_or_else(|err| panic!("{}", err));

    let mut solver = puzzle_definition
        .make_solver(config)
        .unwrap_or_else(|err| panic!("{}", err));

    let solutions = solver.solve();
    let count = solutions.0.len();
    println!("Solutions:\n{}", solutions);
    println!("{} solutions", count);
}
