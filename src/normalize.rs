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

pub fn diagnostics(output: Vec<u8>) -> String {
    let mut from_bytes = String::from_utf8_lossy(&output).to_string();
    from_bytes = from_bytes.replace("\r\n", "\n");

    let mut normalized = String::new();

    for line in from_bytes.lines() {
        if let Some(line) = filter(line) {
            normalized += &line;
            if !normalized.ends_with("\n\n") {
                normalized.push('\n');
            }
        }
    }

    trim(normalized)
}

fn filter(line: &str) -> Option<String> {
    if line.trim_start().starts_with("--> ") {
        if let Some(cut_end) = line.rfind(&['/', '\\'][..]) {
            let cut_start = line.find('>').unwrap() + 2;
            return Some(line[..cut_start].to_owned() + "$DIR/" + &line[cut_end + 1..]);
        }
    }

    if line == "error: aborting due to previous error" {
        return None;
    }

    if line == "To learn more, run the command again with --verbose." {
        return None;
    }

    Some(line.to_owned())
}
