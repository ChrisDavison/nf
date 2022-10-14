use std::collections::HashSet;
use std::ffi::OsStr;
use std::path::Path;

use crate::nfmatch::NFMatch;

pub struct NoteFilter {
    pub all_words: HashSet<String>,
    pub words: HashSet<String>,
    pub tags: HashSet<String>,
    case_sensitive: bool,
}

impl NoteFilter {
    pub fn new(words: &[String], case_sensitive: bool) -> NoteFilter {
        let words = if case_sensitive {
            words.to_vec()
        } else {
            words.iter().map(|w| w.to_lowercase()).collect()
        };
        let (tag_words, content_words): (Vec<_>, Vec<_>) =
            words.iter().partition(|w| w.starts_with('@'));
        let tag_set = tag_words.iter().map(|x| x[1..].to_string()).collect();
        let content_word_set: HashSet<String> =
            content_words.iter().map(|x| x.to_string()).collect();

        NoteFilter {
            all_words: content_word_set
                .iter()
                .chain(&tag_set)
                .map(|x| x.to_string())
                .collect(),
            words: content_word_set,
            tags: tag_set,
            case_sensitive,
        }
    }
    pub fn matches(&self, path: &Path) -> NFMatch {
        // let mut what_matches = HashSet::new();
        NFMatch {
            title: self.title_matches(path),
            tags: self.tags_match(path),
            contents: self.contents_match(path),
            header: self.header_match(path),
        }
    }

    pub fn title_matches(&self, path: &Path) -> bool {
        let stem = path
            .file_stem()
            .unwrap_or_else(|| OsStr::new(""))
            .to_string_lossy();
        self.all_words.iter().any(|w| stem.contains(w))
    }

    pub fn contents_match(&self, path: &Path) -> bool {
        let contents = std::fs::read_to_string(&path).unwrap();
        !self.words.is_empty() && self.words.iter().all(|word| contents.contains(word))
    }

    pub fn header_match(&self, path: &Path) -> bool {
        if self.words.is_empty() {
            return false;
        }
        let contents = std::fs::read_to_string(&path).unwrap();
        for line in contents.lines() {
            let line = if self.case_sensitive {
                line.to_string()
            } else {
                line.to_lowercase()
            };
            if line.starts_with('#') && self.words.iter().all(|word| line.contains(word)) {
                return true;
            }
        }
        false
    }

    pub fn tags_match(&self, path: &Path) -> bool {
        let to_match = &self
            .tags
            .iter()
            .map(|t| {
                if t.starts_with('@') {
                    t.to_string()
                } else {
                    format!("{t}")
                }
            })
            .collect::<Vec<String>>();
        let f = tagsearch::filter::Filter::new(to_match, &[], false);
        let file_tags = tagsearch::utility::get_tags_for_file(&path.to_string_lossy().to_string());

        !self.tags.is_empty() && f.matches(&file_tags)
    }
}
