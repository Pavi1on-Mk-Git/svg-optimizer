use super::iter::EasyIter;

pub(crate) struct IdGenerator {
    base_characters: Vec<char>,
    generated_ids: usize,
    used_ids: Vec<String>,
}

impl IdGenerator {
    pub(crate) fn new(used_ids: Vec<String>) -> Self {
        Self {
            // not using abcdef to avoid conflicts with hex colors in CSS
            base_characters: "ghijklmnopqrstuvwxyzGHIJKLMNOPQRSTUVWXYZ".chars().collect(),
            generated_ids: 0,
            used_ids,
        }
    }
}

impl IdGenerator {
    fn get_id(&mut self) -> String {
        let mut char_ids = vec![];
        let mut remaining_count = self.generated_ids;

        loop {
            char_ids.push(remaining_count % self.base_characters.len());
            remaining_count /= self.base_characters.len();

            if remaining_count == 0 {
                break;
            }
            remaining_count -= 1;
        }

        self.generated_ids += 1;

        char_ids.map_to_vec(|id| self.base_characters[id])
    }
}

impl Iterator for IdGenerator {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let id = self.get_id();
            if !self.used_ids.contains(&id) {
                return Some(id);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use itertools::assert_equal;

    use super::IdGenerator;

    const CHARS_SIZE: usize = 40;

    #[test]
    fn test_id_generation() {
        let mut generator = IdGenerator::new(vec![]);

        assert_eq!(generator.nth(5), Some("l".into()));
        assert_eq!(generator.nth(CHARS_SIZE - 1), Some("lg".into()));
        assert_eq!(generator.nth(CHARS_SIZE.pow(2)), Some("mgg".into()));
        assert_eq!(
            generator.nth(CHARS_SIZE.pow(3) - CHARS_SIZE - 1),
            Some("mZZ".into())
        );
    }

    #[test]
    fn test_id_generation_skip_used() {
        let mut generator = IdGenerator::new(vec!["g".into(), "m".into()]);
        assert_equal(
            generator.by_ref().take(8),
            vec!["h", "i", "j", "k", "l", "n", "o", "p"],
        );
        assert_eq!(generator.nth(CHARS_SIZE * 2 - 10), Some("gh".into()));
    }
}
