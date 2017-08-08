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
    cache: HashMap<Vec<char>, T>,
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
        let hash = anagram_hash(item.anagrammable());
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
