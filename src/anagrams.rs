use std::fmt;
use std::hash::Hash;
use std::collections::HashMap;
use std::clone::Clone;

use gnip_twitter_stream::{Tweet, MinimalTweet};

use filters::is_ascii_letter;
pub use edit_dist::EditDistance;

const ASCII_LOWERCASE_OFFSET: u8 = 97;

/// A trait for types that have some string representation suitable
/// for anagram comparisons.
pub trait AsStr {
    fn as_str(&self) -> &str;
}

/// A trait for types which store anagram candidates.
pub trait Store<K, V> {
    fn remove(&mut self, key: &K);
    fn get_item(&self, key: &K) -> Option<V>;
    fn insert(&mut self, key: K, value: V);
}

/// A trait for types which handle results of anagram search.
pub trait Adapter<T, TE: Tester<T>> {
    fn will_check(&mut self, _item: &T) { }
    fn possible_match(&mut self, _p1: &T, _p2: &T) { }
    fn handle_match(&mut self, p1: &T, p2: &T, hash: &TE::Fingerprint);
}

/// A trait for types which validate potential anagrams.
///
/// A large number of anagrams are overly similar or otherwise disatisfying.
/// This type represents some collection of tests to filter out these less
/// desirable results.
pub trait Tester<T> {
    type Fingerprint: Hash + Eq;
    fn fingerprint(&mut self, s: &T) -> Self::Fingerprint;
    fn is_match(&mut self, p1: &T, p2: &T) -> bool;
}

pub struct SimpleAdapter<T> {
    hits: Vec<(T, T)>,
    seen: usize,
    tested: usize,
}

/// A (hashmap backed) in memory store.
pub struct MemoryStore<K, V>(HashMap<K, V>);

/// A simple tester for ascii text.
#[derive(Debug, Clone, Default)]
pub struct AsciiTester {
    edit_dist: EditDistance,
}

/// Stores a count for each ascii char, in order.
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct AsciiFingerprint([u8; 26]);

impl<T, TE> Adapter<T, TE> for SimpleAdapter<T>
    where T: AsStr + Clone,
          TE: Tester<T>,
{
    fn will_check(&mut self, _item: &T) {
        self.seen += 1;
    }

    fn possible_match(&mut self, _p1: &T, _p2: &T) {
        self.tested += 1;
    }

    fn handle_match(&mut self, p1: &T, p2: &T, _hash: &TE::Fingerprint) {
        self.hits.push((p1.to_owned(), p2.to_owned()));
    }
}

impl<K: Hash + Eq, V> MemoryStore<K, V> {
    pub fn new() -> Self {
        MemoryStore(HashMap::new())
    }
}

impl<K: Hash + Eq, V: Clone> Store<K, V> for MemoryStore<K, V> {
    fn remove(&mut self, key: &K) {
        self.0.remove(key);
    }

    fn get_item(&self, key: &K) -> Option<V> {
        self.0.get(key).map(V::clone)
    }

    fn insert(&mut self, key: K, value: V) {
        self.0.insert(key, value);
    }
}

impl<T: AsStr> Tester<T> for AsciiTester {
    type Fingerprint = AsciiFingerprint;

    fn fingerprint(&mut self, s: &T) -> Self::Fingerprint {
        let mut h: [u8; 26] = [0; 26];
        for c in s.as_str().chars()
            .filter(is_ascii_letter)
            .flat_map(char::to_lowercase) {
                let idx = (c as u8 - ASCII_LOWERCASE_OFFSET) as usize;
                h[idx] += 1;
        }
        AsciiFingerprint(h)
    }

    fn is_match(&mut self, p1: &T, p2: &T) -> bool {
        self.test_distance(p1.as_str(), p2.as_str())
    }
}

impl AsciiTester {
    fn test_distance(&mut self, a: &str, b: &str) -> bool {
        const MIN_DIST: f64 = 0.5;
        if a == b { return false }
        let a1 = lowercase_filtered(a);
        let b1 = lowercase_filtered(b);
        if a1 == b1 { return false }
        let dist = self.edit_dist.distance(&a1, &b1);
        if (dist as f64) / (b.chars().count() as f64) < MIN_DIST {
            return false
        }

        let a = word_split_sort(a);
        let b = word_split_sort(b);
        //assert_eq!(a.len(), b.len(), "{} / {}", a, b);
        let dist = self.edit_dist.distance(&a, &b);
        (dist as f64) / (b.chars().count() as f64) > MIN_DIST
    }
}

impl fmt::Display for AsciiFingerprint {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut result = String::new();
        for (idx, chr) in self.0.iter().enumerate() {
            if *chr == 0 { continue }
            //let count = chr & 511;
            //let chr = ((chr & 127 << 9) >> 9) as u8;
            for _ in 0..*chr {
                result.push((idx as u8 + ASCII_LOWERCASE_OFFSET) as char);
            }
        }
        write!(f, "{}", result)
    }
}

impl AsRef<[u8]> for AsciiFingerprint {
    fn as_ref(&self) -> &[u8] {
        &self.0
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
        for &(ref one, ref two) in self.hits.iter() {
            println!("---------\n{}\n--↕︎--\n{}",
                     one.as_str(),
                     two.as_str());
        }
        println!("saw {} items, found {} anagrams.", self.seen, self.hits.len());
    }
}

/// Handles a single item.
pub fn process_item<T, S, A, TE>(item: T,
                                 store: &mut S,
                                 adapter: &mut A,
                                 tester: &mut TE)
    where T: AsStr,
          S: Store<TE::Fingerprint, T>,
          A: Adapter<T, TE>,
          TE: Tester<T>,
{
    process_item_impl(item, store, adapter, tester, true)
}

/// Checks a new item against stored items, without modifying storage.
/// Mostly exposed for benchmarking.
pub fn check_item<T, S, A, TE>(item: T,
                               store: &mut S,
                               adapter: &mut A,
                               tester: &mut TE)
    where T: AsStr,
          S: Store<TE::Fingerprint, T>,
          A: Adapter<T, TE>,
          TE: Tester<T>,
{
    process_item_impl(item, store, adapter, tester, false)
}

fn process_item_impl<T, S, A, TE>(item: T,
                                  store: &mut S,
                                  adapter: &mut A,
                                  tester: &mut TE,
                                  store_new: bool)
    where T: AsStr,
          S: Store<TE::Fingerprint, T>,
          A: Adapter<T, TE>,
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
            store.remove(&ident);
            return adapter.handle_match(&item, &hit.unwrap(), &ident);
        }
    }

    if store_new {
        store.insert(ident, item)
    }
}

fn lowercase_filtered<T: AsRef<str>>(s: T) -> String {
    s.as_ref().chars()
        .filter(is_ascii_letter)
        .flat_map(char::to_lowercase)
        .collect::<String>()
}

//TODO: this has three allocations too many
/// Given a string, removes ignored chars, lowercases, and sorts by word.
fn word_split_sort<T: AsRef<str>>(s: T) -> String {
    let s = s.as_ref();
    let s: String = s.chars()
        .flat_map(|c| if is_ascii_letter(&c) { c.to_lowercase() } else { ' '.to_lowercase() })
        .collect();
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

impl AsStr for MinimalTweet {
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
        let mut tester = AsciiTester::default();
        let h = tester.fingerprint(&inp);

        assert_eq!(h.to_string(), inp)
    }

    #[test]
    fn word_split() {
        let one = "twenty six\n\n#MissUniverse #Philippines";
        let two = "#MissUniverse #Philippines\n\ntwenty six";
        assert_eq!(word_split_sort(one), word_split_sort(two));
    }

    #[test]
    fn distance() {
        let one = "twenty six\n\n#MissUniverse #Philippines";
        let two = "#MissUniverse #Philippines\n\ntwenty six";
        let mut tester = AsciiTester::default();

        assert!(!tester.test_distance(one, two));
        assert!(!tester.is_match(&one, &two));

        let one = "joji // will he";
        let two = "willhe//joji";
        eprintln!("{} / {}", word_split_sort(one), word_split_sort(two));

        assert!(!tester.test_distance(one, two));
        assert!(!tester.is_match(&one, &two));
    }

    #[test]
    fn integration() {
    let mut adapter = SimpleAdapter::new();
    let mut tester = AsciiTester::default();
    let mut store = MemoryStore::new();
    let one = "twenty six\n\n#MissUniverse #Philippines";
    let two = "#MissUniverse #Philippines\n\ntwenty six";

    process_item(one, &mut store, &mut adapter, &mut tester);
    process_item(two, &mut store, &mut adapter, &mut tester);

    process_item("joji // will he", &mut store, &mut adapter, &mut tester);
    process_item("willhe//joji", &mut store, &mut adapter, &mut tester);

    assert_eq!(adapter.hits.len(), 0);
    }
}
