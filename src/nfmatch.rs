impl std::fmt::Display for NFMatch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut parts = Vec::new();
        parts.push(if self.title { "T" } else { " " });
        parts.push(if self.tags { "t" } else { " " });
        parts.push(if self.contents { "c" } else { " " });
        parts.push(if self.header { "h" } else { " " });
        let joined = parts.join("");
        write!(f, "{joined}")
    }
}

pub struct NFMatch {
    pub title: bool,
    pub tags: bool,
    pub contents: bool,
    pub header: bool,
}

impl NFMatch {
    pub fn no_match(&self) -> bool {
        !(self.title || self.tags || self.contents || self.header)
    }

    pub fn oxford_commaize(&self) -> String {
        let mut parts = Vec::new();
        if self.title {
            parts.push("Title".to_string());
        }
        if self.tags {
            parts.push("Tags".to_string());
        }
        if self.contents {
            parts.push("Contents".to_string());
        }
        if self.header {
            parts.push("Header".to_string());
        }

        if parts.len() > 2 {
            let last = &parts[parts.len() - 1];
            let n = parts.len();
            parts[n - 1] = format!("and {last}");
        }
        parts.join(", ")
    }
}
