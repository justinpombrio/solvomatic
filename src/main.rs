//! See README.md

// TODO:
// - [ ] release 'parser-ll1' at least enough that it can be imported!
// - [x] have letters and digits be disjoint?

use argh::FromArgs;
use parser_ll1::{CompiledParser, Grammar, GrammarError, Parser};
use solvomatic::{Solvomatic, State};
use std::collections::HashMap;
use std::fmt;
use std::fs;
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

fn read_letter(word: &str) -> Option<i32> {
    if word.len() != 1 {
        return None;
    }

    let byte = word.chars().next().unwrap() as i32;
    if byte >= 65 && byte <= 90 {
        // upper case letter -> negative one-based number index
        Some(64 - byte)
    } else if byte >= 97 && byte <= 122 {
        // lower case letter -> negative one-based number index
        Some(96 - byte)
    } else {
        None
    }
}

impl FromStr for Entry {
    type Err = BadInput;

    fn from_str(word: &str) -> Result<Entry, BadInput> {
        if word == "." {
            Ok(Entry(None))
        } else if word == "*" {
            // '*' shouldn't occur in actual input, only templates
            Ok(Entry(Some(STAR)))
        } else if let Some(n) = read_letter(word) {
            Ok(Entry(Some(n)))
        } else if let Ok(n) = u32::from_str(word) {
            Ok(Entry(Some(n as i32)))
        } else {
            Err(BadInput::BadEntry(word.to_owned()))
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
    /// Whitespace that occurs around and between the entries. Has length one
    /// more than the number of entries.
    whitespace: Vec<String>,
}

impl Layout {
    fn new(input: &str) -> Layout {
        let mut whitespace = Vec::new();

        let mut remaining_input = input;
        loop {
            match remaining_input.find("*") {
                None => {
                    whitespace.push(remaining_input.to_owned());
                    return Layout { whitespace };
                }
                Some(offset) => {
                    whitespace.push(remaining_input[..offset].to_owned());
                    remaining_input = &remaining_input[offset + 1..];
                }
            }
        }
    }

    fn parse_input<'a>(
        &'a self,
        input: &'a str,
    ) -> impl Iterator<Item = Result<Entry, BadInput>> + 'a {
        LayoutParser {
            at_start: true,
            whitespace: &self.whitespace,
            input,
        }
    }
}

#[derive(Debug, Default)]
struct LayoutParser<'a> {
    at_start: bool,
    whitespace: &'a [String],
    input: &'a str,
}

impl<'a> Iterator for LayoutParser<'a> {
    type Item = Result<Entry, BadInput>;

    fn next(&mut self) -> Option<Result<Entry, BadInput>> {
        // If we're at the end return None, or error if there's stuff left over.
        if self.whitespace.is_empty() {
            if !self.input.is_empty() {
                let line = self.input.lines().next().unwrap().to_string();
                return Some(Err(BadInput::DoesNotMatchLayout(
                    line,
                    "too much input".to_owned(),
                )));
            }
            return None;
        }

        // If we're at the beginning, consume the initial whitespace
        if self.at_start {
            let (whitespace, rest) = self.whitespace.split_first().unwrap();
            self.whitespace = rest;
            if !self.input.starts_with(whitespace) {
                let line = self.input.lines().next().unwrap_or("[empty]").to_string();
                return Some(Err(BadInput::DoesNotMatchLayout(
                    line,
                    format!("expected '{}'", whitespace),
                )));
            }
            self.input = &self.input[whitespace.len()..];
            self.at_start = false;
        }

        // Consume word and subsequent whitespace
        let (whitespace, rest) = self.whitespace.split_first().unwrap();
        self.whitespace = rest;
        let word = if whitespace == "" {
            let word = &self.input[0..1];
            self.input = &self.input[1..];
            word
        } else if let Some(offset) = self.input.find(whitespace) {
            let word = &self.input[0..offset];
            self.input = &self.input[offset + whitespace.len()..];
            word
        } else {
            let line = self.input.lines().next().unwrap_or("").to_string();
            return Some(Err(BadInput::DoesNotMatchLayout(
                line,
                format!("expected '{}'", whitespace),
            )));
        };

        // Parse Entry
        Some(Entry::from_str(word))
    }
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
            entries: vec![Entry(None); layout.whitespace.len() - 1],
            layout: layout.clone(),
        }
    }
}

impl Data {
    fn new(input: &str, layout: Arc<Layout>) -> Result<Data, BadInput> {
        Ok(Data {
            entries: layout.parse_input(input).collect::<Result<Vec<_>, _>>()?,
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

        write!(f, "{}", self.layout.whitespace[0])?;
        for i in 0..self.entries.len() {
            write!(
                f,
                "{:>padding$}{}{}",
                "",
                &entries[i],
                self.layout.whitespace[i + 1],
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
    BadEntry(String),
    DoesNotMatchLayout(String, String),
    BadRangeEntry(i32),
}

impl fmt::Display for BadInput {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            BadInput::BadEntry(word) => write!(f, "Bad entry '{}'", word),
            BadInput::DoesNotMatchLayout(bad_part, message) => {
                write!(f, "Bad input at '{}' ({})", bad_part, message)
            }
            BadInput::BadRangeEntry(entry) => write!(f, "Bad range entry {}", entry),
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
                let mut key_to_var_list = HashMap::new();
                let data = Data::new(data, layout.clone())?;
                for (i, entry) in data.entries.iter().enumerate() {
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
                            .or_insert_with(|| Vec::new())
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
        .span(|span| read_letter(span.substr).unwrap());
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
        .map(|entry| PuzzleRule::Sum(entry));
    let prod_p = entry_p
        .clone()
        .preceded(g.string("prod")?)
        .map(|entry| PuzzleRule::Prod(entry));
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

    solver.solve().unwrap_or_else(|err| panic!("{}", err));
    println!("{}", solver.display_table());
}
