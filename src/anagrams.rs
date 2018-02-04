use std::fmt;
use std::hash::Hash;
use std::collections::HashMap;

use edit_distance::edit_distance;
use gnip_twitter_stream::Tweet;

use filters::is_ascii_letter;

const ASCII_LOWERCASE_OFFSET: u8 = 97;

/// A trait for types that have some string representation suitable
/// for anagram comparisons.
pub trait AsStr {
    fn as_str(&self) -> &str;
}

/// A trait for types which store anagram candidates.
pub trait Store<K, V> {
    fn get_item(&self, key: &K) -> Option<&V>;
    fn insert(&mut self, key: K, value: V);
}

/// A trait for types which handle results of anagram search.
pub trait Adapter<T> {
    fn will_check(&mut self, _item: &T) { }
    fn possible_match(&mut self, _p1: &T, _p2: &T) { }
    fn handle_match(&mut self, p1: &T, p2: &T);
}

/// A trait for types which validate potential anagrams.
///
/// A large number of anagrams are overly similar or otherwise disatisfying.
/// This type represents some collection of tests to filter out these less
/// desirable results.
pub trait Tester<T> {
    type Fingerprint: Hash + Eq;
    fn fingerprint(&mut self, s: &T) -> Self::Fingerprint;
    fn is_match(&self, p1: &T, p2: &T) -> bool;
}

pub struct SimpleAdapter<T> {
    hits: Vec<(T, T)>,
    seen: usize,
    tested: usize,
}

/// A (hashmap backed) in memory store.
pub struct MemoryStore<K, V>(HashMap<K, V>);

/// A simple tester for ascii text.
pub struct AsciiTester;

/// Stores an ascii char and a count as a single u16.
///
/// This makes our hash 52 stack bytes
///
/// Storing the actual char is currently redundant, because it can be determined
/// from the index; however space savings would be possible by using `SmallVec`.
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct AsciiFingerprint([u16; 26]);

impl<T: AsStr + Clone> Adapter<T> for SimpleAdapter<T> {
    fn will_check(&mut self, _item: &T) {
        self.seen += 1;
    }

    fn possible_match(&mut self, _p1: &T, _p2: &T) {
        self.tested += 1;
    }

    fn handle_match(&mut self, p1: &T, p2: &T) {
        self.hits.push((p1.to_owned(), p2.to_owned()));
    }
}

impl<K: Hash + Eq, V> MemoryStore<K, V> {
    pub fn new() -> Self {
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

impl<T: AsStr> Tester<T> for AsciiTester {
    type Fingerprint = AsciiFingerprint;

    fn fingerprint(&mut self, s: &T) -> Self::Fingerprint {
        let mut h: [u16; 26] = [0; 26];
        for c in s.as_str().chars()
            .filter(is_ascii_letter)
            .flat_map(char::to_lowercase) {
                let b = c as u16;
                let idx = (b - ASCII_LOWERCASE_OFFSET as u16) as usize;
                if h[idx] == 0 {
                    h[idx] = b << 9;
                }
                h[idx] += 1;
        }
        AsciiFingerprint(h)
    }

    fn is_match(&self, p1: &T, p2: &T) -> bool {
        test_distance(p1.as_str(), p2.as_str())
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


impl<T: AsStr> SimpleAdapter<T> {
    pub fn new() -> Self {
        SimpleAdapter {
            hits: Vec::new(),
            seen: 0,
            tested: 0,
        }
    }

    pub fn print_results(&self) {
        println!("saw {} items, found {} anagrams.", self.seen, self.hits.len());
        for &(ref one, ref two) in self.hits.iter() {
            println!("---------\n{}\n--↕︎--\n{}",
                     one.as_str(),
                     two.as_str());
        }
    }
}

/// Handles a single item.
pub fn process_item<T, S, A, TE>(item: T,
                                 store: &mut S,
                                 adapter: &mut A,
                                 tester: &mut TE)
    where T: AsStr,
          S: Store<TE::Fingerprint, T>,
          A: Adapter<T>,
          TE: Tester<T>,
{
    let ident = tester.fingerprint(&item);
    adapter.will_check(&item);
    {
        let hit = store.get_item(&ident);
        let is_hit = match store.get_item(&ident) {
            Some(ref hit) => {
                adapter.possible_match(&item, hit);
                tester.is_match(&item, hit)
            }
            None => false,
        };

        if is_hit {
            return adapter.handle_match(&item, hit.unwrap());
        }
    }

    store.insert(ident, item)
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

#[allow(dead_code)]
pub fn anagram_hash(s: &str) -> Vec<char> {
    let mut out = s.chars().filter(is_ascii_letter)
        .flat_map(char::to_lowercase)
        .collect::<Vec<char>>();
    out.sort();
    out
}

impl AsStr for String {
    fn as_str(&self) -> &str {
        &self
    }
}

impl<'a> AsStr for &'a str {
    fn as_str(&self) -> &str {
        &self
    }
}

impl AsStr for Tweet {
    fn as_str(&self) -> &str {
        &self.text
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn cleverness() {
        let inp = "aabbccddeefffffghiz";
        let mut tester = AsciiTester;
        let h = tester.fingerprint(&inp);

        assert_eq!(h.to_string(), inp)
    }
}
