use super::FACES;
use gotoo_pixel_engine::TextInputEvent;
use nucleo_matcher::{
    Config, Matcher,
    pattern::{AtomKind, CaseMatching, Normalization, Pattern},
};

pub struct Search {
    pub query: String,
    pub matches: Vec<usize>,
    pub active: bool,
    cursor: usize,
    select_all: bool,
    matcher: Matcher,
}
impl Search {
    pub fn new() -> Self {
        let mut search = Self {
            query: String::new(),
            matches: Vec::new(),
            active: true,
            cursor: 0,
            select_all: false,
            matcher: Matcher::new(Config::DEFAULT),
        };
        search.refresh();
        search
    }
    pub fn clear(&mut self) {
        self.query.clear();
        self.cursor = 0;
        self.select_all = false;
        self.refresh();
    }
    pub fn select_all(&mut self) {
        self.select_all = true;
    }
    pub fn display(&self) -> String {
        // Keep the caret visible without changing the query or its match semantics.
        let before: String = self.query[..self.cursor]
            .chars()
            .rev()
            .take(17)
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect();
        let after: String = self.query[self.cursor..]
            .chars()
            .take(17_usize.saturating_sub(before.chars().count()))
            .collect();
        format!("{before}{}{after}", if self.active { "|" } else { "" })
    }
    pub fn edit(&mut self, events: &[TextInputEvent]) {
        for event in events {
            match event {
                TextInputEvent::Insert(text) => {
                    if self.select_all {
                        self.query.clear();
                        self.cursor = 0;
                    }
                    let text: String = text
                        .chars()
                        .filter(|c| !c.is_control())
                        .take(80_usize.saturating_sub(self.query.chars().count()))
                        .collect();
                    self.query.insert_str(self.cursor, &text);
                    self.cursor += text.len();
                }
                TextInputEvent::Backspace | TextInputEvent::Delete if self.select_all => {
                    self.query.clear();
                    self.cursor = 0;
                }
                TextInputEvent::Backspace if self.cursor > 0 => {
                    let previous = self.query[..self.cursor].char_indices().last().unwrap().0;
                    self.query.drain(previous..self.cursor);
                    self.cursor = previous;
                }
                TextInputEvent::Delete if self.cursor < self.query.len() => {
                    self.query.remove(self.cursor);
                }
                TextInputEvent::Left if self.cursor > 0 => {
                    self.cursor = self.query[..self.cursor].char_indices().last().unwrap().0;
                }
                TextInputEvent::Right if self.cursor < self.query.len() => {
                    self.cursor += self.query[self.cursor..].chars().next().unwrap().len_utf8();
                }
                TextInputEvent::Home => self.cursor = 0,
                TextInputEvent::End => self.cursor = self.query.len(),
                _ => {}
            }
            self.select_all = false;
        }
        self.refresh();
    }
    fn refresh(&mut self) {
        let mut alphabet: Vec<_> = (0..FACES.len()).collect();
        alphabet.sort_by_key(|&i| FACES[i].0.to_lowercase());
        if self.query.trim().is_empty() {
            self.matches = alphabet;
            return;
        }
        let pattern = Pattern::new(
            &self.query,
            CaseMatching::Ignore,
            Normalization::Smart,
            AtomKind::Fuzzy,
        );
        let scored = pattern.match_list(alphabet.iter().map(|&i| FACES[i].0), &mut self.matcher);
        self.matches = scored
            .iter()
            .map(|(name, _)| FACES.iter().position(|(n, _)| n == name).unwrap())
            .collect();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn fuzzy_abbreviation_ranks_and_empty_query_restores_alphabet() {
        let mut search = Search::new();
        search.edit(&[TextInputEvent::Insert("pfd".into())]);
        assert_eq!(FACES[search.matches[0]].0, "Playfair Display");
        search.clear();
        assert_eq!(search.matches.len(), FACES.len());
        assert!(
            search
                .matches
                .windows(2)
                .all(|w| FACES[w[0]].0.to_lowercase() <= FACES[w[1]].0.to_lowercase())
        );
        search.edit(&[TextInputEvent::Insert("zzzzzz".into())]);
        assert!(search.matches.is_empty());
    }
    #[test]
    fn unicode_edits_and_replace_selection_are_ordered() {
        let mut search = Search::new();
        search.edit(&[
            TextInputEvent::Insert("a\u{e9}b".into()),
            TextInputEvent::Left,
            TextInputEvent::Backspace,
        ]);
        assert_eq!(search.query, "ab");
        search.edit(&[TextInputEvent::Delete]);
        assert_eq!(search.query, "a");
        search.select_all();
        search.edit(&[TextInputEvent::Insert("Work Sans".into())]);
        assert_eq!(FACES[search.matches[0]].0, "Work Sans");
    }
}
