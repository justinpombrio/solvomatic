//! FILL

// TODO:
// - [ ] release 'parser-ll1' at least enough that it can be imported!
// - [ ] have letters and digits be disjoint?

#![feature(slice_take)]

use parser_ll1::{CompiledParser, Grammar, GrammarError, Parser};
use solvomatic::{Solvomatic, State};
use std::collections::HashMap;
use std::fmt;
use std::fs;
use std::rc::Rc;
use std::str::FromStr;

const DICTIONARY_PATH: &str = "/usr/share/dict/words";

/************************
 *     Entry            *
 ************************/

/// One entry: either a letter or a number, or '.' if unknown.
#[derive(Debug, Clone, Copy)]
struct Entry(Option<i32>);

// Assumes word.len() == 1!
fn read_letter(word: &str) -> Option<i32> {
    if word.len() != 1 {
        return None;
    }

    let byte = word.chars().next().unwrap() as i32;
    if byte >= 65 && byte <= 90 {
        Some(byte - 64)
    } else if byte >= 97 && byte <= 122 {
        Some(byte - 96)
    } else {
        None
    }
}

impl Entry {
    /// If `negative_letters` is true, parse `a, b, c` as `-1, -2, -3`
    fn read(word: &str, negative_letters: bool) -> Option<Entry> {
        if word == "." {
            Some(Entry(None))
        } else if word == "*" {
            // Shouldn't occur in actual input, only templates
            Some(Entry(Some(0)))
        } else if let Some(n) = read_letter(word) {
            if negative_letters {
                Some(Entry(Some(-n)))
            } else {
                Some(Entry(Some(n)))
            }
        } else if let Ok(num) = i32::from_str(word) {
            Some(Entry(Some(num)))
        } else {
            None
        }
    }

    fn write(&self, f: &mut fmt::Formatter, is_letter: bool) -> fmt::Result {
        if let Some(num) = self.0 {
            if is_letter && num >= 1 && num <= 26 {
                write!(f, "{}", (num + 64) as u8 as char)
            } else {
                write!(f, "{}", num)
            }
        } else {
            write!(f, ".")
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
    /// Map from entry index to boolean indicating whether it should be interpreted
    /// as a letter vs. as a numeral.
    is_letter: Vec<bool>,
}

impl Layout {
    fn new(input: &str) -> Layout {
        let mut whitespace = Vec::new();

        let mut remaining_input = input;
        loop {
            match remaining_input.find("*") {
                None => {
                    whitespace.push(remaining_input.to_owned());
                    return Layout {
                        is_letter: vec![false; whitespace.len() - 1],
                        whitespace,
                    };
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
        // If true, parse 'a, b, c' as '-1, -2, -3'.
        // Used hackily to help parse rules, which use both letters & numbers.
        negative_letters: bool,
    ) -> impl Iterator<Item = Result<Entry, BadInput>> + 'a {
        LayoutParser {
            at_start: true,
            whitespace: &self.whitespace,
            input,
            negative_letters,
        }
    }
}

#[derive(Debug, Default)]
struct LayoutParser<'a> {
    at_start: bool,
    whitespace: &'a [String],
    input: &'a str,
    negative_letters: bool,
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
            let whitespace = self.whitespace.take_first().unwrap();
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
        let whitespace = self.whitespace.take_first().unwrap();
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
        match Entry::read(word, self.negative_letters) {
            Some(entry) => Some(Ok(entry)),
            None => Some(Err(BadInput::BadEntry(word.to_owned()))),
        }
    }
}

/************************
 *     Data             *
 ************************/

#[derive(Debug, Default)]
struct Data {
    entries: Vec<Entry>,
    layout: Rc<Layout>,
}

impl State for Data {
    type Var = usize;
    type Value = i32;
    type MetaData = Rc<Layout>;

    fn set(&mut self, var: usize, val: i32) {
        self.entries[var] = Entry(Some(val));
    }

    fn new(layout: &Rc<Layout>) -> Data {
        Data {
            entries: vec![Entry(None); layout.is_letter.len()],
            layout: layout.clone(),
        }
    }
}

impl Data {
    fn new(input: &str, layout: Rc<Layout>, negative_letters: bool) -> Result<Data, BadInput> {
        Ok(Data {
            entries: layout
                .parse_input(input, negative_letters)
                .collect::<Result<Vec<_>, _>>()?,
            layout,
        })
    }
}

impl fmt::Display for Data {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.layout.whitespace[0])?;
        for i in 0..self.entries.len() {
            self.entries[i].write(f, self.layout.is_letter[i])?;
            write!(f, "{}", self.layout.whitespace[i + 1])?;
        }
        Ok(())
    }
}

/************************
 *     Bad Input Error  *
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
    min: i32,
    max: i32,
    data: String,
}

#[derive(Debug, Clone)]
enum PuzzleRule {
    Sum(i32),
    Prod(i32),
    Word,
}

#[derive(Debug, Clone)]
struct PuzzleRuleAndDatas {
    rule: PuzzleRule,
    datas: Vec<String>,
}

#[derive(Debug)]
struct PuzzleDefinition {
    layout: String,
    ranges: Vec<PuzzleRange>,
    rules: Vec<PuzzleRuleAndDatas>,
    initial: Option<String>,
}

impl PuzzleDefinition {
    fn make_solver(self) -> Result<Solvomatic<Data>, BadInput> {
        use solvomatic::constraints::{Pred, Prod, Seq, Sum};

        fn load_word_list(word_len: usize) -> Seq<i32> {
            // TODO: loading the same file repeatedly!
            let words = fs::read_to_string(DICTIONARY_PATH).expect("Failed to load dictionary");
            let words = words
                .lines()
                .map(|s| s.trim())
                .filter(|s| &s.to_lowercase() == s)
                .filter(|s| s.chars().count() == word_len)
                .map(|s| s.chars().map(|ch| ch as i32 - 94).collect::<Vec<_>>())
                .collect::<Vec<_>>();
            Seq::new(word_len, words)
        }

        let original_layout = Layout::new(&self.layout);
        let layout = Rc::new(original_layout.clone());
        let mut solver = Solvomatic::new(layout.clone());

        for range in &self.ranges {
            let data = Data::new(&range.data, layout.clone(), false)?;
            for (i, entry) in data.entries.iter().enumerate() {
                match entry {
                    Entry(None) => (),
                    Entry(Some(0)) => solver.var(i, range.min..=range.max),
                    Entry(Some(n)) => return Err(BadInput::BadRangeEntry(*n)),
                }
            }
        }

        for rule in &self.rules {
            let mut var_lists = Vec::new();
            for data in &rule.datas {
                let mut key_to_var_list = HashMap::new();
                let data = Data::new(data, layout.clone(), true /* parse letters as neg */)?;
                for (i, entry) in data.entries.iter().enumerate() {
                    if let Entry(Some(entry)) = entry {
                        let key = if *entry >= 0 {
                            // entry was a number
                            None
                        } else {
                            // entry was a letter
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
                match rule.rule {
                    PuzzleRule::Sum(sum) => solver.constraint(var_list, Sum::new(sum)),
                    PuzzleRule::Prod(prod) => solver.constraint(var_list, Prod::new(prod)),
                    PuzzleRule::Word => {
                        let words = load_word_list(var_list.len());
                        solver.constraint(var_list, words);
                    }
                }
            }
        }

        if let Some(initial) = self.initial {
            let data = Data::new(&initial, layout.clone(), false)?;
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

        // TODO
        Ok(solver)
    }
}

fn make_puzzle_parser() -> Result<impl CompiledParser<PuzzleDefinition>, GrammarError> {
    use parser_ll1::{choice, tuple};

    // Whitespace includes '#' comments that extend to the end of the line
    let mut g = Grammar::with_whitespace("([ \t\n]+|#[^\n]*\n)+")?;

    // A data section is a sequence of lines that all start with '|'
    let data_p = g.regex("data", "(\\|[^\n]*\n)+")?.span(|span| {
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

    // layout
    //   DATA
    let layout_p = tuple("layout", (g.string("layout")?, data_p.clone())).map(|(_, data)| data);

    // range min max
    //   DATA
    //   ...
    let range_p = tuple(
        "range",
        (
            g.string("range")?,
            entry_p.clone(),
            entry_p.clone(),
            data_p.clone(),
        ),
    )
    .map(|(_, min, max, data)| PuzzleRange { min, max, data });

    // rule name arg...
    //   DATA
    //   ...
    let sum_p = entry_p
        .clone()
        .preceded(g.string("sum")?)
        .map(|entry| PuzzleRule::Sum(entry));
    let prod_p = entry_p
        .clone()
        .preceded(g.string("prod")?)
        .map(|entry| PuzzleRule::Prod(entry));
    let word_p = g.string("word")?.constant(PuzzleRule::Word);
    let rule_p = choice("rule name ('sum', 'prod', 'word')", (sum_p, prod_p, word_p));
    let rule_and_datas_p = tuple("rule", (g.string("rule")?, rule_p, data_p.clone().many1()))
        .map(|(_, rule, datas)| PuzzleRuleAndDatas { rule, datas });

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
            range_p.many1(),
            rule_and_datas_p.many1(),
            initial_p.opt(),
        ),
    )
    .map(|(layout, ranges, rules, initial)| PuzzleDefinition {
        layout,
        ranges,
        rules,
        initial,
    });

    // Better error messages
    g.regex("identifier", "[a-zA-Z]+")?;

    g.compile_parser(puzzle_p)
}

fn main() {
    let filename = "examples/textual/palindrome.txt";

    let parser = make_puzzle_parser().unwrap_or_else(|err| panic!("{}", err));
    let file_contents = fs::read_to_string(filename).unwrap();
    let puzzle_definition = parser
        .parse(filename, &file_contents)
        .unwrap_or_else(|err| panic!("{}", err));
    println!("{:#?}", puzzle_definition);
    let mut solver = puzzle_definition
        .make_solver()
        .unwrap_or_else(|err| panic!("{}", err));
    solver.solve().unwrap_or_else(|err| panic!("{}", err));
    println!("{}", solver.table_display());
    println!("ok");
}
