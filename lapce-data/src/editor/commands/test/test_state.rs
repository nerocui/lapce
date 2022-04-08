//! Representation of the visible editor state.

use std::{
    collections::HashMap,
    fmt::{Debug, Display},
};

use crate::movement::{SelRegion, Selection};
use lazy_static::lazy_static;
use regex::{Captures, Regex};

#[derive(PartialEq)]
pub struct TestState {
    pub contents: String,
    pub selection: Selection,
}

impl Debug for TestState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut contents = self.contents.clone();
        let mut inserted = 0;
        for (id, region) in self.selection.regions().iter().enumerate() {
            let marker = format!("<${id}>");
            contents.insert_str(region.start() + inserted, &marker);
            inserted += marker.len();
        }
        for (id, region) in self.selection.regions().iter().enumerate() {
            if !region.is_caret() {
                let marker = format!("</${id}>");
                contents.insert_str(region.end() + inserted, &marker);
                inserted += marker.len();
            }
        }

        f.write_str(&contents)
    }
}

impl Display for TestState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(self, f)
    }
}

impl PartialEq<TestState> for &str {
    fn eq(&self, other: &TestState) -> bool {
        other.to_string() == *self
    }
}

impl TestState {
    pub fn parse(initial: &str) -> Self {
        lazy_static! {
            static ref START: Regex = Regex::new(r#"<\$(\d+)>"#).unwrap();
            static ref END: Regex = Regex::new(r#"</\$(\d+)>"#).unwrap();
        }

        let mut starts = HashMap::new();
        let mut ends = HashMap::new();

        let mut removed = 0;

        let mut contents = initial.to_string();

        let mut record_cursor_marker =
            |captures: Captures, map: &mut HashMap<usize, usize>| {
                let whole_match = captures.get(0).unwrap();
                let id_match = captures.get(1).unwrap();

                let cursor_id = id_match.as_str().parse::<usize>().unwrap();

                let start = whole_match.start() - removed;
                let end = whole_match.end() - removed;
                let marker_len = end - start;

                map.insert(cursor_id, start)
                    .map(|_| panic!("Duplicate cursor marker: {whole_match:?}"));

                unsafe { contents.as_mut_vec() }.drain(start..end);

                removed += marker_len;
            };

        for start in START.captures_iter(initial) {
            record_cursor_marker(start, &mut starts);
        }

        for end in END.captures_iter(initial) {
            record_cursor_marker(end, &mut ends);
        }

        let mut selection = Selection::new();
        for (id, start) in starts.into_iter() {
            let region = if let Some(end) = ends.get(&id).copied() {
                SelRegion::new(start, end, None)
            } else {
                SelRegion::caret(start)
            };
            selection.add_region(region)
        }

        Self {
            contents,
            selection,
        }
    }
}

mod test_state_tests {
    use super::*;

    #[test]
    fn can_parse_single_cursor() {
        let text = r#"foo<$0>bar"#;

        let state = TestState::parse(text);
        assert_eq!(1, state.selection.len());
        assert_eq!("foobar", state.contents);
    }

    #[test]
    fn can_parse_multiple_cursors() {
        let text = r#"foo<$0>b<$1>ar"#;

        let state = TestState::parse(text);
        assert_eq!(2, state.selection.len());
        assert_eq!("foobar", state.contents);
    }

    #[test]
    fn can_parse_single_selection() {
        let text = r#"foo<$0>bar</$0>"#;

        let state = TestState::parse(text);
        assert_eq!(1, state.selection.len());
        assert_eq!("foobar", state.contents);
    }

    #[test]
    fn can_format_into_string() {
        let text = r#"fo<$0>o<$1>bar</$1>"#;

        let state = TestState::parse(text);
        assert_eq!("foobar", state.contents);

        assert_eq!(text, state.to_string());
    }
}