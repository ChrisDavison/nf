use clap::Parser;
use std::collections::HashSet;
use std::ffi::OsStr;
use std::path::Path;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
#[clap(propagate_version = true)]
struct App {
    /// Words to search for
    words: Vec<String>,
    /// Output in format suitable for vimgrep
    #[clap(short, long)]
    vimgrep: bool,
}

fn main() {
    let app = App::parse();

    let filter = NoteFilter::new(&app.words);

    let files = tagsearch::utility::get_files(None).unwrap_or_default();
    for filename in files {
        let p = Path::new(&filename);
        let nfmatch = filter.matches(p);
        if nfmatch == NFMatch::NoMatch {
            continue;
        }
        if app.vimgrep {
            println!("{}:1:{}", p.to_string_lossy(), nfmatch.oxford_commaize());
        } else {
            println!("{} {:60}", nfmatch, p.to_string_lossy());
        }
    }
}

struct NoteFilter {
    all_words: HashSet<String>,
    words: HashSet<String>,
    tags: HashSet<String>,
}

#[derive(PartialEq)]
enum NFMatch {
    NoMatch,
    OnlyTitle,
    OnlyTags,
    OnlyContents,
    TitleTags,
    TitleContents,
    TagsContents,
    TitleTagsContents,
}

impl std::fmt::Display for NFMatch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NFMatch::NoMatch => write!(f, "   "),
            NFMatch::OnlyTitle => write!(f, "T  "),
            NFMatch::OnlyTags => write!(f, " t "),
            NFMatch::OnlyContents => write!(f, "  c"),
            NFMatch::TitleTags => write!(f, "Tt "),
            NFMatch::TitleContents => write!(f, "T c"),
            NFMatch::TagsContents => write!(f, " tc"),
            NFMatch::TitleTagsContents => write!(f, "Ttc"),
        }
    }
}

impl NFMatch {
    fn oxford_commaize(&self) -> &str {
        match self {
            NFMatch::NoMatch => "",
            NFMatch::OnlyTitle => "Title only",
            NFMatch::OnlyTags => "Tags only",
            NFMatch::OnlyContents => "Contents only",
            NFMatch::TitleTags => "Title and tags",
            NFMatch::TitleContents => "Title and contents",
            NFMatch::TagsContents => "Tags and contents",
            NFMatch::TitleTagsContents => "Title, tags, and contents",
        }
    }
}

impl NoteFilter {
    pub fn new(words: &[String]) -> NoteFilter {
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
        }
    }
    pub fn matches(&self, path: &Path) -> NFMatch {
        // let mut what_matches = HashSet::new();
        match (
            self.title_matches(path),
            self.tags_match(path),
            self.contents_match(path),
        ) {
            (false, false, false) => NFMatch::NoMatch,
            (true, false, false) => NFMatch::OnlyTitle,
            (false, true, false) => NFMatch::OnlyTags,
            (false, false, true) => NFMatch::OnlyContents,
            (true, true, false) => NFMatch::TitleTags,
            (true, false, true) => NFMatch::TitleContents,
            (false, true, true) => NFMatch::TagsContents,
            (true, true, true) => NFMatch::TitleTagsContents,
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

    pub fn tags_match(&self, path: &Path) -> bool {
        let to_match = &self.tags.iter().cloned().collect::<Vec<String>>();
        let f = tagsearch::filter::Filter::new(to_match, &[], false);
        let file_tags = tagsearch::utility::get_tags_for_file(&path.to_string_lossy().to_string());

        !self.tags.is_empty() && f.matches(&file_tags)
    }
}
