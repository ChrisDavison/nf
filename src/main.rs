use clap::Parser;
use std::path::Path;

mod notefilter;

use notefilter::NoteFilter;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
#[clap(propagate_version = true)]
struct App {
    /// Words to search for
    words: Vec<String>,
    /// Output in format suitable for vimgrep
    #[clap(short, long)]
    vimgrep: bool,
    /// Case-sensitive
    #[clap(short, long)]
    sensitive: bool,
}

fn main() {
    let app = App::parse();

    let filter = NoteFilter::new(&app.words, app.sensitive, app.vimgrep);

    let files = tagsearch::utility::get_files(None)
        .unwrap_or_default();

    for filename in files {
        let path = Path::new(&filename);
        let fm = filter.matches(path);
        if !fm.matches.is_empty() {
            println!("{}", fm);
        }
    }
}
