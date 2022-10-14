use clap::Parser;
use std::path::Path;

mod nfmatch;
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

    let filter = NoteFilter::new(&app.words, app.sensitive);

    let files = tagsearch::utility::get_files(None).unwrap_or_default();
    for filename in files {
        let p = Path::new(&filename);
        let nfmatch = filter.matches(p);
        if nfmatch.no_match() {
            continue;
        }
        if app.vimgrep {
            println!("{}:1:1:{}", p.to_string_lossy(), nfmatch.oxford_commaize());
        } else {
            println!("{} {:60}", nfmatch, p.to_string_lossy());
        }
    }
}
