use std::collections::HashSet;
use std::ffi::OsStr;
use std::path::Path;

use regex::Regex;

type Position = (usize, usize);

enum Displayer {
    Plain,
    Vimgrep
}

pub struct FileMatch {
    filename: String,
    shortstr: String,
    pub matches: Vec<(Position, NFMatch)>,
    displaytype: Displayer
}

impl std::fmt::Display for FileMatch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.displaytype {
            Displayer::Plain => write!(f, "{} {}", self.shortstr, self.filename),
            Displayer::Vimgrep => {
                let mut lines = Vec::new();
                for (pos, nfm) in &self.matches[..] {
                    let (pref, msg): (String, String) = match nfm {
                        NFMatch::Title(m) =>  ("Title ".into(), m.to_string()),
                        NFMatch::Tags(m) => ("@ ".into(), m.to_string()),
                        NFMatch::Contents(m) => ("".into(), m.to_string()),
                        NFMatch::Header(m) => ("".into(), m.to_string()),
                    };
                    lines.push(format!("{}:{}:{}:{}{}", self.filename, pos.0, pos.1, pref, msg));
                }
                write!(f, "{}", lines.join("\n"))
            },
        }
    }
}

#[derive(Clone)]
pub enum NFMatch {
    Title(String),
    Tags(String),
    Contents(String),
    Header(String),
}


pub struct NoteFilter {
    pub all_words: HashSet<String>,
    pub words: HashSet<String>,
    word_regex: Regex,
    pub tags: Vec<String>,
    case_sensitive: bool,
    as_vimgrep: bool,
}

impl NoteFilter {
    pub fn new(words: &[String], case_sensitive: bool, as_vimgrep: bool) -> NoteFilter {
        let words = if case_sensitive {
            words.to_vec()
        } else {
            words.iter().map(|w| w.to_lowercase()).collect()
        };
        let (tag_words, content_words): (Vec<_>, Vec<_>) =
            words.iter().partition(|w| w.starts_with('@'));
        let tag_set: HashSet<String> = tag_words.iter().map(|x| x[1..].to_string()).collect();
        let content_word_set: HashSet<String> =
            content_words.iter().map(|x| x.to_string()).collect();
        let word_regex: Regex = Regex::new(&words.join(r"\s+")).unwrap();

        NoteFilter {
            all_words: content_word_set
                .iter()
                .chain(&tag_set)
                .map(|x| x.to_string())
                .collect(),
            words: content_word_set,
            word_regex,
            tags: tag_set.iter().map(String::from).collect::<Vec<String>>(),
            case_sensitive,
            as_vimgrep,
        }
    }

    pub fn matches(&self, path: &Path) -> FileMatch {
        let mut matches = Vec::new();
        let mut shortstr = [b' ', b' ', b' ', b' '];
        let t = self.title_matches(path);
        if !t.is_empty() {
        //     // matches.extend(t);
            shortstr[0] = b't';
        }
        let t = self.tags_match(path);
        if !t.is_empty() {
            matches.extend(t);
            shortstr[1] = b'@';
        }
        let h = self.header_match(path);
        if !h.is_empty() {
            matches.extend(h);
            shortstr[2] = b'h';
        }
        let c = self.contents_match(path);
        if !c.is_empty() {
            matches.extend(c);
            shortstr[3] = b'c';
        }
        let shortstr = String::from_utf8_lossy(&shortstr).to_string();
        FileMatch {
            filename: path.to_string_lossy().to_string(),
            shortstr,
            matches,
            displaytype: if self.as_vimgrep { Displayer::Vimgrep } else { Displayer::Plain }
        }
    }

    pub fn title_matches(&self, path: &Path) -> Vec<(Position, NFMatch)> {
        let stem = path
            .file_stem()
            .unwrap_or_else(|| OsStr::new(""))
            .to_string_lossy().to_string();
        if self.word_regex.is_match(&stem) {
            vec![((1, 1), NFMatch::Title(stem))]
        } else {
            Vec::new()
        }
    }

    pub fn contents_match(&self, path: &Path) -> Vec<(Position, NFMatch)> {
        let contents = std::fs::read_to_string(&path).unwrap();

        let mut matches = Vec::new();
        if self.words.is_empty() {
            return matches;
        }
        for (i, line) in contents.lines().enumerate() {
            for m in self.word_regex.find_iter(line) {
                matches.push(((i+1, m.start()+1), NFMatch::Contents(line.to_string())))
            }
        }
        matches
    }

    pub fn header_match(&self, path: &Path) -> Vec<(Position, NFMatch)> {
        let mut matching_headers: Vec<_> = Vec::new();
        if self.words.is_empty() {
            return matching_headers;
        }
        let contents = std::fs::read_to_string(&path).unwrap();

        for (i, line) in contents.lines().enumerate() {
            let line = if self.case_sensitive {
                line.to_string()
            } else {
                line.to_lowercase()
            };
            if line.starts_with('#') && self.word_regex.is_match(&line) {
                matching_headers.push(((i+1, 1), NFMatch::Header(line)));
            }
        }
        matching_headers
    }

    pub fn tags_match(&self, path: &Path) -> Vec<(Position, NFMatch)> {
        let contents = std::fs::read_to_string(&path).unwrap();

        let f = tagsearch::filter::Filter::new(&self.tags, &[], false);

        let mut matches = Vec::new();
        for (i, line) in contents.lines().enumerate() {
            let linetags = tagsearch::utility::get_tags_from_string(line);
            if !self.tags.is_empty() && f.matches(&linetags) {
                matches.push(((i+1, 1), NFMatch::Tags(line.into())))
            }
        }
        matches
    }
}
