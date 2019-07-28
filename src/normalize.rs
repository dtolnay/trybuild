pub fn trim<S: AsRef<[u8]>>(output: S) -> String {
    let bytes = output.as_ref();
    let mut normalized = String::from_utf8_lossy(bytes).to_string();

    let len = normalized.trim_end().len();
    normalized.truncate(len);

    if !normalized.is_empty() {
        normalized.push('\n');
    }

    normalized
}

pub fn diagnostics(output: &std::process::Output) -> Variations {
    let mut err_from_bytes = String::from_utf8_lossy(&output.stderr).to_string();
    err_from_bytes = err_from_bytes.replace("\r\n", "\n");

    let mut out_from_bytes = String::from_utf8_lossy(&output.stdout).to_string();
    variations_out = out_from_bytes.replace("\r\n", "\n");

    let variations_err = [Basic, StripCouldNotCompile]
        .iter()
        .map(|normalization| apply(&err_from_bytes, *normalization))
        .collect();

    Variations {
        variations_err,
        variations_out,
    }
}

pub struct Variations {
    variations_err: Vec<String>,
    variations_out: String,
}

impl Variations {
    pub fn map<F: FnMut(String) -> String>(self, f: F) -> Self {
        Variations {
            variations_err: self.variations_err.into_iter().map(f).collect(),
            variations_out: self.variations_out,
        }
    }

    pub fn preferred(&self) -> &str {
        self.variations_err.last().unwrap()
    }

    pub fn any<F: FnMut(&str) -> bool>(&self, mut f: F) -> bool {
        self.variations_err.iter().any(|stderr| f(stderr))
    }

    pub fn stdout(&self) -> &Vec<String> {
        &self.variations_out
    }
}

#[derive(PartialOrd, PartialEq, Copy, Clone)]
enum Normalization {
    Basic,
    StripCouldNotCompile,
}

use self::Normalization::*;

fn apply(original: &str, normalization: Normalization) -> String {
    let mut normalized = String::new();

    for line in original.lines() {
        if let Some(line) = filter(line, normalization) {
            normalized += &line;
            if !normalized.ends_with("\n\n") {
                normalized.push('\n');
            }
        }
    }

    trim(normalized)
}

fn filter(line: &str, normalization: Normalization) -> Option<String> {
    if line.trim_start().starts_with("--> ") {
        if let Some(cut_end) = line.rfind(&['/', '\\'][..]) {
            let cut_start = line.find('>').unwrap() + 2;
            return Some(line[..cut_start].to_owned() + "$DIR/" + &line[cut_end + 1..]);
        }
    }

    if line.starts_with("error: aborting due to ") {
        return None;
    }

    if line == "To learn more, run the command again with --verbose." {
        return None;
    }

    if normalization >= StripCouldNotCompile {
        if line.starts_with("error: Could not compile `") {
            return None;
        }
    }

    Some(line.to_owned())
}
