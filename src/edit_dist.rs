#[derive(Debug, Clone, Default)]
pub struct EditDistance {
    storage: Vec<usize>,
}

impl EditDistance {
    // substantially based on https://github.com/febeling/edit-distance,
    // but with much fewer allocations
    pub fn distance<A, B>(&mut self, a: A, b: B) -> usize
        where A: AsRef<str>,
              B: AsRef<str>,
    {
        let a = a.as_ref();
        let b = b.as_ref();
        let a_count = a.chars().count();
        let b_count = b.chars().count();
        let nb_cols = b_count + 1;

        // resize storage if needed
        let nb_items = (a_count + 1) * (b_count + 1);

        if self.storage.len() < nb_items {
            let to_add = nb_items - self.storage.len();
            self.storage.reserve(to_add);
            for _ in 0..to_add {
                self.storage.push(0);
            }
        }

        // initial values
        for i in 0..a_count {
            let idx = row_col_to_idx(i, 0, nb_cols);
            let idx1 = row_col_to_idx(i+1, 0, nb_cols);
            self.storage[idx1] = self.storage[idx] + 1;
        }

        for i in 0..b_count {
            self.storage[i+1] = self.storage[i] + 1;
        }

        for (i, ca) in a.chars().enumerate() {
            for (j, cb) in b.chars().enumerate() {
                let alternatives = [
                    // deletion
                    self.storage[row_col_to_idx(i, j+1, nb_cols)] + 1,
                    // insertion
                    self.storage[row_col_to_idx(i+1, j, nb_cols)] + 1,
                    // match or substitution
                    self.storage[row_col_to_idx(i, j, nb_cols)] + if ca == cb { 0 } else { 1 }];
                let min_alt = *alternatives.iter().min().unwrap();
                self.storage[row_col_to_idx(i+1, j+1, nb_cols)] = min_alt;
            }
        }
        self.storage[row_col_to_idx(a_count, b_count, nb_cols)]
    }
}

#[inline(always)]
fn row_col_to_idx(row: usize, col: usize, col_nb: usize) -> usize {
    row * col_nb + col
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smoke_test() {
        let mut ed = EditDistance::default();
        // manually calculated from a reference implementation
        let tests = [
            ("hello", "bellow", 2),
            ("my friend", "remains", 7),
            ("this", "that", 2),
            ("heaven", "is a place on earth", 16),
            ("a fundamentally", "non-creative person", 17),
            ("is writing", "these test cases", 13),
        ];

        for &(a, b, exp) in tests.iter() {
            assert_eq!(ed.distance(a, b), exp);
        }
    }
}
