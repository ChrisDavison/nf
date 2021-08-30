use clap::{crate_version, App, Arg};
use std::collections::HashSet;
use std::path::Path;

fn main() {
    let matches = App::new("NoteFind")
        .about("Search note titles, contents, and tags")
        .version(crate_version!())
        .arg(Arg::with_name("word").multiple(true).required(true))
        .get_matches();

    let words: Vec<String> = matches
        .values_of("word")
        .unwrap()
        .map(|x| x.to_owned())
        .collect();

    let filter = NoteFilter::new(&words);

    let files = tagsearch::utility::get_files(None).unwrap();
    for filename in files {
        let p = Path::new(&filename);
        let matches = filter.matches(p);
        if matches.is_empty() {
            continue;
        }
        let parts = vec![
            if matches.contains("title") { "T" } else { " " },
            if matches.contains("tags") { "t" } else { " " },
            if matches.contains("contents") {
                "c"
            } else {
                " "
            },
        ]
        .join("");
        println!("{} {:60}", parts, p.to_string_lossy(),);
    }
}

struct NoteFilter {
    all_words: HashSet<String>,
    words: HashSet<String>,
    tags: HashSet<String>,
}

impl NoteFilter {
    pub fn new(words: &[String]) -> NoteFilter {
        let (tag_words, content_words): (Vec<_>, Vec<_>) =
            words.iter().partition(|w| w.starts_with('@'));
        let tag_set = tag_words.iter().map(|x| x[1..].to_string()).collect();
        let content_word_set: HashSet<String> =
            content_words.iter().map(|x| x.to_string()).collect();

        NoteFilter {
            all_words: content_word_set.iter().chain(&tag_set).map(|x| x.to_string()).collect(),
            words: content_word_set,
            tags: tag_set,
        }
    }
    pub fn matches(&self, path: &Path) -> HashSet<String> {
        let mut what_matches = HashSet::new();
        if self.title_matches(path) {
            what_matches.insert(String::from("title"));
        }
        if self.contents_match(path) {
            what_matches.insert(String::from("contents"));
        }
        if self.tags_match(path) {
            what_matches.insert(String::from("tags"));
        }
        what_matches
    }

    pub fn title_matches(&self, path: &Path) -> bool {
        let stem = path.file_stem().unwrap().to_string_lossy();
        self.all_words.iter().any(|w| stem.contains(w))
    }

    pub fn contents_match(&self, path: &Path) -> bool {
        let contents = std::fs::read_to_string(&path).unwrap();
        !self.words.is_empty() && self.words.iter().all(|word| contents.contains(word))
    }

    pub fn tags_match(&self, path: &Path) -> bool {
        let file_tags = tagsearch::utility::get_tags_for_file(&path.to_string_lossy().to_string());
        !self.tags.is_empty() && self.tags.iter().all(|t| file_tags.contains(t))
    }
}
