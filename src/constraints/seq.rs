use super::{Constraint, YesNoMaybe};
use bitvec::{bitvec, vec::BitVec};
use std::fmt::Debug;
use std::fs;
use std::hash::Hash;
use std::path::Path;

/// The constraint that `{X1, ..., Xn}` is a word from a list of allowed words. Or more generally,
/// that that sequence is present in a list of allowed sequences.
#[derive(Debug, Clone)]
pub struct Seq<T: Debug + Hash + Eq + Ord + Clone + Sized + Send + Sync + 'static> {
    #[allow(unused)]
    seq_len: usize,
    allowed_seqs: Vec<Vec<T>>,
}

impl Seq<char> {
    /// Allowed sequences are the words of the given length from the file at `path`.
    pub fn word_list_file(
        path: impl AsRef<Path>,
        word_len: usize,
    ) -> Result<Seq<char>, std::io::Error> {
        let word_list = fs::read_to_string(path)?;
        let allowed_words = word_list
            .lines()
            .map(|s| s.trim())
            .map(|s| s.to_lowercase())
            .filter(|s| s.chars().count() == word_len)
            .map(|s| s.chars().collect::<Vec<_>>())
            .collect::<Vec<_>>();
        for word in &allowed_words {
            assert_eq!(word.len(), word_len);
        }
        Ok(Seq {
            seq_len: word_len,
            allowed_seqs: allowed_words,
        })
    }
}

impl<T: Debug + Hash + Eq + Ord + Clone + Sized + Send + Sync + 'static> Seq<T> {
    /// Constraint that the sequence is one of the sequences listed in `allowed_seqs`.
    pub fn new(seq_len: usize, allowed_seqs: impl IntoIterator<Item = Vec<T>>) -> Seq<T> {
        let allowed_seqs = allowed_seqs.into_iter().collect::<Vec<_>>();
        for seq in &allowed_seqs {
            assert_eq!(seq.len(), seq_len);
        }
        Seq {
            seq_len,
            allowed_seqs,
        }
    }
}

/// Represents a set of sequences.
#[derive(Debug, Clone)]
pub struct SeqSet {
    /// `set[i]` iff `allowed_seqs[i]` _may be_ in the set.
    /// (`true` means `Yes|Maybe`, `false` means `No`)
    set: BitVec,
    /// The number of sequences in the set.
    count: u128,
}

impl<T: Debug + Hash + Eq + Ord + Clone + Sized + Send + Sync + 'static> Constraint<T> for Seq<T> {
    // Represents a set of possible words. Set[i] iff self.allowed_seqs[i] is in the set.
    type Set = SeqSet;

    const NAME: &'static str = "Seq";

    fn singleton(&self, index: usize, elem: T) -> Self::Set {
        let mut set: BitVec = bitvec![0; self.allowed_seqs.len()];
        for (i, seq) in self.allowed_seqs.iter().enumerate() {
            if seq[index] == elem {
                set.set(i, true);
            }
        }
        SeqSet { set, count: 1 }
    }

    fn and(&self, a: Self::Set, b: Self::Set) -> Self::Set {
        SeqSet {
            set: a.set & b.set,
            count: a.count * b.count,
        }
    }

    fn or(&self, a: Self::Set, b: Self::Set) -> Self::Set {
        SeqSet {
            set: a.set | b.set,
            count: a.count + b.count,
        }
    }

    fn check(&self, set: Self::Set) -> YesNoMaybe {
        use YesNoMaybe::{Maybe, No, Yes};

        let num_words = set.set.count_ones() as u128;
        if num_words == 0 {
            No
        } else if num_words == set.count {
            Yes
        } else {
            Maybe
        }
    }
}

#[test]
fn test_seq() {
    use YesNoMaybe::{Maybe, No, Yes};

    let s = Seq::word_list_file("/usr/share/dict/words", 3).unwrap();

    // Three words of the form `s_x`: `s{a,e,i,o}x`.
    assert_eq!(
        s.and(s.singleton(0, 's'), s.singleton(2, 'x'))
            .set
            .count_ones(),
        4
    );

    assert_eq!(
        s.check(s.and(
            s.and(s.singleton(1, 'o'), s.singleton(2, 'o')),
            s.singleton(0, 't')
        )),
        Yes
    );

    assert_eq!(
        s.check(s.and(
            s.and(s.singleton(1, 'o'), s.singleton(2, 'o')),
            s.or(s.singleton(0, 't'), s.singleton(0, 'b'))
        )),
        Yes
    );

    assert_eq!(
        s.check(s.and(
            s.and(
                s.singleton(1, 'o'),
                s.or(s.singleton(2, 'o'), s.singleton(2, 'x'))
            ),
            s.or(s.singleton(0, 't'), s.singleton(0, 'b'))
        )),
        Maybe
    );

    assert_eq!(
        s.check(s.and(
            s.singleton(0, 'x'),
            s.and(s.singleton(1, 'a'), s.singleton(2, 'c'))
        )),
        No
    );

    assert_eq!(
        s.check(s.and(
            s.or(s.singleton(0, 't'), s.singleton(0, 'n')),
            s.and(
                s.or(s.singleton(1, 't'), s.singleton(1, 'n')),
                s.or(s.singleton(2, 't'), s.singleton(2, 'n'))
            )
        )),
        No
    );

    assert_eq!(
        s.check(s.and(
            s.and(s.singleton(0, 't'), s.singleton(1, 't')),
            s.singleton(2, 't')
        )),
        No
    );
}
