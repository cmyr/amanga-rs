use std::fmt;
use std::collections::HashMap;

use edit_distance::edit_distance;
use gnip_twitter_stream::Tweet;

use filters::is_ascii_letter;

/// A trait for types that have some string representation suitable
/// for anagram comparisons.
pub trait Anagrammable: Clone {
    fn anagrammable(&self) -> &str;
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

pub struct AnagramFinder<T> {
    cache: HashMap<AnagramHash, T>,
    hits: Vec<(T, T)>,
    seen: usize,
}

impl<T: Anagrammable> AnagramFinder<T> {
    pub fn new() -> Self {
        AnagramFinder {
            cache: HashMap::new(),
            hits: Vec::new(),
            seen: 0,
        }
    }
    pub fn add(&mut self, item: &T) {
        self.seen += 1;
        let hash = AnagramHash::new(item.anagrammable());
        let exists = self.cache.contains_key(&hash);

        if exists {
            let to_test = self.cache.remove(&hash).unwrap();
            if test_distance(item.anagrammable(), to_test.anagrammable()) {
                self.hits.push((item.to_owned(), to_test));
                return
            }
        }
        self.cache.insert(hash, item.to_owned());
    }

    pub fn print_results(&self) {
        println!("saw {} items, found {} anagrams.", self.seen, self.hits.len());
        for &(ref one, ref two) in self.hits.iter() {
            println!("---------\n{}\n--↕︎--\n{}", one.anagrammable(), two.anagrammable());
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

/// Stores an ascii char and a count as a single u16.
///
/// This makes our hash 52 stack bytes
///
/// Storing the actual char is currently redundant, because it can be determined
/// from the index; however space savings would be possible by using `SmallVec`.
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
struct AnagramHash([u16; 26]);

impl AnagramHash {
    fn new(s: &str) -> Self {
        let mut h: [u16; 26] = [0; 26];
        for c in s.chars().filter(is_ascii_letter).flat_map(char::to_lowercase) {
            let b = c as u16;
            let idx = (b - ASCII_LOWERCASE_OFFSET as u16) as usize;
            if h[idx] == 0 {
                h[idx] = b << 9;
            }
            h[idx] += 1;
        }
        AnagramHash(h)
    }
}

impl fmt::Display for AnagramHash {
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

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn cleverness() {
        let inp = "aabbccddeefffffghiz";
        let h = AnagramHash::new(inp);
        assert_eq!(&h.to_string(), inp)
    }
}
