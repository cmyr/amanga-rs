use std::fmt;
use std::hash::Hash;
use std::collections::HashMap;

use edit_distance::edit_distance;
use gnip_twitter_stream::Tweet;

use filters::is_ascii_letter;

/// A trait for types that have some string representation suitable
/// for anagram comparisons.
//TODO: replace with AsRef<str>
pub trait Anagrammable: Clone {
    fn anagrammable(&self) -> &str;
}

pub trait Store<K, V> {
    fn get_item(&self, key: &K) -> Option<&V>;
    fn insert(&mut self, key: K, value: V);
}

pub trait Adapter<T> {
    fn possible_match(&mut self, _hit: &T) { }
    fn handle_match(&mut self, p1: &T, p2: &T);
}

//TODO: combine tester + fingerprinter
pub trait Tester<T> {
    fn is_match(&self, p1: &T, p2: &T) -> bool;
}

pub trait Fingerprinter {
    type Fingerprint: Hash + Eq;
    fn fingerprint(&mut self, s: &str) -> Self::Fingerprint;
}

pub struct SimpleAdapter<T> {
    hits: Vec<(T, T)>,
    seen: usize,
}

struct MemoryStore<K, V>(HashMap<K, V>);

struct AsciiTester;

struct AsciiFingerprinter;

/// Stores an ascii char and a count as a single u16.
///
/// This makes our hash 52 stack bytes
///
/// Storing the actual char is currently redundant, because it can be determined
/// from the index; however space savings would be possible by using `SmallVec`.
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
struct AsciiFingerprint([u16; 26]);

impl<T: Anagrammable> Adapter<T> for SimpleAdapter<T> {
    fn possible_match(&mut self, _hit: &T) {
        self.seen += 1;
    }

    fn handle_match(&mut self, p1: &T, p2: &T) {
        self.hits.push((p1.to_owned(), p2.to_owned()));
    }
}

pub fn find_anagrams<T, S, ST, A, TE, F>(source: &mut S,
                                         store: &mut ST,
                                         adapter: &mut A,
                                         tester: &TE,
                                         fingerp: &mut F)
    where T: Anagrammable,
          S: Iterator<Item=T>,
          ST: Store<F::Fingerprint, T>,
          A: Adapter<T>,
          TE: Tester<T>,
          F: Fingerprinter,
{
    for item in source {
        let ident = fingerp.fingerprint(item.anagrammable());
        {
            let hit = store.get_item(&ident);
            let is_hit = match store.get_item(&ident) {
                Some(ref hit) => {
                    adapter.possible_match(hit);
                    tester.is_match(&item, hit)
                }
                None => false,
            };

            if is_hit {
                adapter.handle_match(&item, hit.unwrap());
                continue
            }
        }
        store.insert(ident, item)
    }
}

pub fn simple_find_anagrams<T, S, A>(source: &mut S, adapter: &mut A)
    where T: Anagrammable,
          S: Iterator<Item=T>,
          A: Adapter<T>,
{
    let mut fingerp = AsciiFingerprinter;
    let mut store = MemoryStore::new();
    let tester = AsciiTester;

    find_anagrams(source, &mut store, adapter, &tester, &mut fingerp)
}

impl<K: Hash + Eq, V> MemoryStore<K, V> {
    fn new() -> Self {
        MemoryStore(HashMap::new())
    }
}

impl<K: Hash + Eq, V> Store<K, V> for MemoryStore<K, V> {
    fn get_item(&self, key: &K) -> Option<&V> {
        self.0.get(key)
    }

    fn insert(&mut self, key: K, value: V) {
        self.0.insert(key, value);
    }
}

impl<T: Anagrammable> Tester<T> for AsciiTester {
    fn is_match(&self, p1: &T, p2: &T) -> bool {
        test_distance(p1.anagrammable(), p2.anagrammable())
    }
}


impl<T: Anagrammable> SimpleAdapter<T> {
    pub fn new() -> Self {
        SimpleAdapter {
            hits: Vec::new(),
            seen: 0,
        }
    }

    pub fn print_results(&self) {
        println!("saw {} items, found {} anagrams.", self.seen, self.hits.len());
        for &(ref one, ref two) in self.hits.iter() {
            println!("---------\n{}\n--↕︎--\n{}",
                     one.anagrammable(),
                     two.anagrammable());
        }
    }
}

pub fn test_distance(s1: &str, s2: &str) -> bool {
    const MIN_DIST: f64 = 0.5;
    let dist = edit_distance(&lowercase_filtered(s1), &lowercase_filtered(s2));
    if (dist as f64) / (s2.chars().count() as f64) < MIN_DIST {
        return false
    }

    let s1 = word_split_sort(s1);
    let s2 = word_split_sort(s2);
    //assert_eq!(s1.len(), s2.len(), "{} / {}", s1, s2);
    let dist = edit_distance(&s1, &s2);
    (dist as f64) / (s2.chars().count() as f64) > MIN_DIST
}

fn lowercase_filtered<T: AsRef<str>>(s: T) -> String {
    s.as_ref().chars()
        .filter(is_ascii_letter)
        .flat_map(char::to_lowercase)
        .collect::<String>()
}

/// Given a string, removes ignored chars, lowercases, and sorts by word.
fn word_split_sort<T: AsRef<str>>(s: T) -> String {
    let s = s.as_ref();
    let mut words = s.split_whitespace()
        .map(|s| {
            s.chars()
                .filter(is_ascii_letter)
                .flat_map(char::to_lowercase)
                .collect::<String>()
        })
    .collect::<Vec<_>>();
    words.sort();
    words.as_slice().join(" ")
}

pub fn anagram_hash(s: &str) -> Vec<char> {
    let mut out = s.chars().filter(is_ascii_letter)
        .flat_map(char::to_lowercase)
        .collect::<Vec<char>>();
    out.sort();
    out
}

const ASCII_LOWERCASE_OFFSET: u8 = 97;

impl Fingerprinter for AsciiFingerprinter {
    type Fingerprint = AsciiFingerprint;
    fn fingerprint(&mut self, s: &str) -> Self::Fingerprint {
        let mut h: [u16; 26] = [0; 26];
        for c in s.chars().filter(is_ascii_letter).flat_map(char::to_lowercase) {
            let b = c as u16;
            let idx = (b - ASCII_LOWERCASE_OFFSET as u16) as usize;
            if h[idx] == 0 {
                h[idx] = b << 9;
            }
            h[idx] += 1;
        }
        AsciiFingerprint(h)
    }
}

impl fmt::Display for AsciiFingerprint {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut result = String::new();
        for chr in &self.0 {
            if *chr == 0 { continue }
            let count = chr & 511;
            let chr = ((chr & 127 << 9) >> 9) as u8;
            for _ in 0..count {
                result.push(chr as char);
            }
        }
        write!(f, "{}", result)
    }
}

impl Anagrammable for String {
    fn anagrammable(&self) -> &str {
        &self
    }
}

impl Anagrammable for Tweet {
    fn anagrammable(&self) -> &str {
        &self.text
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn cleverness() {
        let inp = "aabbccddeefffffghiz";
        let mut ascii_f = AsciiFingerprinter;
        let h = ascii_f.fingerprint(inp);

        assert_eq!(&h.to_string(), inp)
    }
}
