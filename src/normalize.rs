#[cfg(test)]
#[path = "tests.rs"]
mod tests;

use self::Normalization::*;
use crate::directory::Directory;
use crate::run::PathDependency;
use std::cmp;
use std::mem;
use std::path::Path;

#[derive(Copy, Clone)]
pub(crate) struct Context<'a> {
    pub krate: &'a str,
    pub source_dir: &'a Directory,
    pub workspace: &'a Directory,
    pub input_file: &'a Path,
    pub target_dir: &'a Directory,
    pub path_dependencies: &'a [PathDependency],
}

macro_rules! normalizations {
    ($($name:ident,)*) => {
        #[derive(PartialOrd, PartialEq, Copy, Clone)]
        enum Normalization {
            $($name,)*
        }

        impl Normalization {
            const ALL: &'static [Self] = &[$($name),*];
        }

        impl Default for Variations {
            fn default() -> Self {
                Variations {
                    variations: [$(($name, String::new()).1),*],
                }
            }
        }
    };
}

normalizations! {
    Basic,
    StripCouldNotCompile,
    StripCouldNotCompile2,
    StripForMoreInformation,
    StripForMoreInformation2,
    TrimEnd,
    RustLib,
    TypeDirBackslash,
    WorkspaceLines,
    PathDependencies,
    CargoRegistry,
    ArrowOtherCrate,
    RelativeToDir,
    LinesOutsideInputFile,
    Unindent,
    AndOthers,
    StripLongTypeNameFiles,
    UnindentAfterHelp,
    AndOthersVerbose,
    UnindentMultilineNote,
    // New normalization steps are to be inserted here at the end so that any
    // snapshots saved before your normalization change remain passing.
}

/// For a given compiler output, produces the set of saved outputs against which
/// the compiler's output would be considered correct. If the test's saved
/// stderr file is identical to any one of these variations, the test will pass.
///
/// This is a set rather than just one normalized output in order to avoid
/// breaking existing tests when introducing new normalization steps. Someone
/// may have saved stderr snapshots with an older version of trybuild, and those
/// tests need to continue to pass with newer versions of trybuild.
///
/// There is one "preferred" variation which is what we print when the stderr
/// file is absent or not a match.
pub(crate) fn diagnostics(output: &str, context: Context) -> Variations {
    let output = output.replace("\r\n", "\n");

    let mut result = Variations::default();
    for (i, normalization) in Normalization::ALL.iter().enumerate() {
        result.variations[i] = apply(&output, *normalization, context);
    }

    result
}

pub(crate) struct Variations {
    variations: [String; Normalization::ALL.len()],
}

impl Variations {
    pub fn preferred(&self) -> &str {
        self.variations.last().unwrap()
    }

    pub fn any<F: FnMut(&str) -> bool>(&self, mut f: F) -> bool {
        self.variations.iter().any(|stderr| f(stderr))
    }

    pub fn concat(&mut self, other: &Self) {
        for (this, other) in self.variations.iter_mut().zip(&other.variations) {
            if !this.is_empty() && !other.is_empty() {
                this.push('\n');
            }
            this.push_str(other);
        }
    }
}

pub(crate) fn trim<S: AsRef<[u8]>>(output: S) -> String {
    let bytes = output.as_ref();
    let mut normalized = String::from_utf8_lossy(bytes).into_owned();

    let len = normalized.trim_end().len();
    normalized.truncate(len);

    if !normalized.is_empty() {
        normalized.push('\n');
    }

    normalized
}

fn apply(original: &str, normalization: Normalization, context: Context) -> String {
    let mut normalized = String::new();

    let lines: Vec<&str> = original.lines().collect();
    let mut filter = Filter {
        all_lines: &lines,
        normalization,
        context,
        hide_numbers: 0,
        other_types: None,
    };
    for i in 0..lines.len() {
        if let Some(line) = filter.apply(i) {
            normalized += &line;
            if !normalized.ends_with("\n\n") {
                normalized.push('\n');
            }
        }
    }

    normalized = unindent(normalized, normalization);

    trim(normalized)
}

struct Filter<'a> {
    all_lines: &'a [&'a str],
    normalization: Normalization,
    context: Context<'a>,
    hide_numbers: usize,
    other_types: Option<usize>,
}

impl<'a> Filter<'a> {
    fn apply(&mut self, index: usize) -> Option<String> {
        let mut line = self.all_lines[index].to_owned();

        if self.hide_numbers > 0 {
            hide_leading_numbers(&mut line);
            self.hide_numbers -= 1;
        }

        let trim_start = line.trim_start();
        let indent = line.len() - trim_start.len();
        let prefix = if trim_start.starts_with("--> ") {
            Some("--> ")
        } else if trim_start.starts_with("::: ") {
            Some("::: ")
        } else {
            None
        };

        if prefix == Some("--> ") && self.normalization < ArrowOtherCrate {
            if let Some(cut_end) = line.rfind(&['/', '\\'][..]) {
                let cut_start = indent + 4;
                line.replace_range(cut_start..cut_end + 1, "$DIR/");
                return Some(line);
            }
        }

        if prefix.is_some() {
            line = line.replace('\\', "/");
            let line_lower = line.to_ascii_lowercase();
            let target_dir_pat = self
                .context
                .target_dir
                .to_string_lossy()
                .to_ascii_lowercase()
                .replace('\\', "/");
            let source_dir_pat = self
                .context
                .source_dir
                .to_string_lossy()
                .to_ascii_lowercase()
                .replace('\\', "/");
            let mut other_crate = false;
            if line_lower.find(&target_dir_pat) == Some(indent + 4) {
                let mut offset = indent + 4 + target_dir_pat.len();
                let mut out_dir_crate_name = None;
                while let Some(slash) = line[offset..].find('/') {
                    let component = &line[offset..offset + slash];
                    if component == "out" {
                        if let Some(out_dir_crate_name) = out_dir_crate_name {
                            let replacement = format!("$OUT_DIR[{}]", out_dir_crate_name);
                            line.replace_range(indent + 4..offset + 3, &replacement);
                            other_crate = true;
                            break;
                        }
                    } else if component.len() > 17
                        && component.rfind('-') == Some(component.len() - 17)
                        && is_ascii_lowercase_hex(&component[component.len() - 16..])
                    {
                        out_dir_crate_name = Some(&component[..component.len() - 17]);
                    } else {
                        out_dir_crate_name = None;
                    }
                    offset += slash + 1;
                }
            } else if let Some(i) = line_lower.find(&source_dir_pat) {
                if self.normalization >= RelativeToDir && i == indent + 4 {
                    line.replace_range(i..i + source_dir_pat.len(), "");
                    if self.normalization < LinesOutsideInputFile {
                        return Some(line);
                    }
                    let input_file_pat = self
                        .context
                        .input_file
                        .to_string_lossy()
                        .to_ascii_lowercase()
                        .replace('\\', "/");
                    if line_lower[i + source_dir_pat.len()..].starts_with(&input_file_pat) {
                        // Keep line numbers only within the input file (the
                        // path passed to our `fn compile_fail`. All other
                        // source files get line numbers erased below.
                        return Some(line);
                    }
                } else {
                    line.replace_range(i..i + source_dir_pat.len() - 1, "$DIR");
                    if self.normalization < LinesOutsideInputFile {
                        return Some(line);
                    }
                }
                other_crate = true;
            } else {
                let workspace_pat = self
                    .context
                    .workspace
                    .to_string_lossy()
                    .to_ascii_lowercase()
                    .replace('\\', "/");
                if let Some(i) = line_lower.find(&workspace_pat) {
                    line.replace_range(i..i + workspace_pat.len() - 1, "$WORKSPACE");
                    other_crate = true;
                }
            }
            if self.normalization >= PathDependencies && !other_crate {
                for path_dep in self.context.path_dependencies {
                    let path_dep_pat = path_dep
                        .normalized_path
                        .to_string_lossy()
                        .to_ascii_lowercase()
                        .replace('\\', "/");
                    if let Some(i) = line_lower.find(&path_dep_pat) {
                        let var = format!("${}", path_dep.name.to_uppercase().replace('-', "_"));
                        line.replace_range(i..i + path_dep_pat.len() - 1, &var);
                        other_crate = true;
                        break;
                    }
                }
            }
            if self.normalization >= RustLib && !other_crate {
                if let Some(pos) = line.find("/rustlib/src/rust/src/") {
                    // --> /home/.rustup/toolchains/nightly/lib/rustlib/src/rust/src/libstd/net/ip.rs:83:1
                    // --> $RUST/src/libstd/net/ip.rs:83:1
                    line.replace_range(indent + 4..pos + 17, "$RUST");
                    other_crate = true;
                } else if let Some(pos) = line.find("/rustlib/src/rust/library/") {
                    // --> /home/.rustup/toolchains/nightly/lib/rustlib/src/rust/library/std/src/net/ip.rs:83:1
                    // --> $RUST/std/src/net/ip.rs:83:1
                    line.replace_range(indent + 4..pos + 25, "$RUST");
                    other_crate = true;
                } else if line[indent + 4..].starts_with("/rustc/")
                    && line
                        .get(indent + 11..indent + 51)
                        .is_some_and(is_ascii_lowercase_hex)
                    && line[indent + 51..].starts_with("/library/")
                {
                    // --> /rustc/c5c7d2b37780dac1092e75f12ab97dd56c30861e/library/std/src/net/ip.rs:83:1
                    // --> $RUST/std/src/net/ip.rs:83:1
                    line.replace_range(indent + 4..indent + 59, "$RUST");
                    other_crate = true;
                }
            }
            if self.normalization >= CargoRegistry && !other_crate {
                if let Some(pos) = line
                    .find("/registry/src/github.com-")
                    .or_else(|| line.find("/registry/src/index.crates.io-"))
                {
                    let hash_start = pos + line[pos..].find('-').unwrap() + 1;
                    let hash_end = hash_start + 16;
                    if line
                        .get(hash_start..hash_end)
                        .is_some_and(is_ascii_lowercase_hex)
                        && line[hash_end..].starts_with('/')
                    {
                        // --> /home/.cargo/registry/src/github.com-1ecc6299db9ec823/serde_json-1.0.64/src/de.rs:2584:8
                        // --> $CARGO/serde_json-1.0.64/src/de.rs:2584:8
                        line.replace_range(indent + 4..hash_end, "$CARGO");
                        other_crate = true;
                    }
                }
            }
            if other_crate && self.normalization >= WorkspaceLines {
                // Blank out line numbers for this particular error since rustc
                // tends to reach into code from outside of the test case. The
                // test stderr shouldn't need to be updated every time we touch
                // those files.
                hide_trailing_numbers(&mut line);
                self.hide_numbers = 1;
                while let Some(next_line) = self.all_lines.get(index + self.hide_numbers) {
                    match next_line.trim_start().chars().next().unwrap_or_default() {
                        '0'..='9' | '|' | '.' => self.hide_numbers += 1,
                        _ => break,
                    }
                }
            }
            return Some(line);
        }

        if line.starts_with("error: aborting due to ") {
            return None;
        }

        if line == "To learn more, run the command again with --verbose." {
            return None;
        }

        if trim_start.starts_with("= note: this compiler was built on 2")
            && trim_start.ends_with("; consider upgrading it if it is out of date")
        {
            return None;
        }

        if self.normalization >= StripCouldNotCompile {
            if line.starts_with("error: Could not compile `") {
                return None;
            }
        }

        if self.normalization >= StripCouldNotCompile2 {
            if line.starts_with("error: could not compile `") {
                return None;
            }
        }

        if self.normalization >= StripForMoreInformation {
            if line.starts_with("For more information about this error, try `rustc --explain") {
                return None;
            }
        }

        if self.normalization >= StripForMoreInformation2 {
            if line.starts_with("Some errors have detailed explanations:") {
                return None;
            }
            if line.starts_with("For more information about an error, try `rustc --explain") {
                return None;
            }
        }

        if self.normalization >= TrimEnd {
            line.truncate(line.trim_end().len());
        }

        if self.normalization >= TypeDirBackslash {
            if line
                .trim_start()
                .starts_with("= note: required because it appears within the type")
            {
                line = line.replace('\\', "/");
            }
        }

        if self.normalization >= AndOthers {
            let trim_start = line.trim_start();
            if trim_start.starts_with("and ") && line.ends_with(" others") {
                let indent = line.len() - trim_start.len();
                let num_start = indent + "and ".len();
                let num_end = line.len() - " others".len();
                if num_start < num_end
                    && line[num_start..num_end].bytes().all(|b| b.is_ascii_digit())
                {
                    line.replace_range(num_start..num_end, "$N");
                }
            }
        }

        if self.normalization >= StripLongTypeNameFiles {
            let trimmed_line = line.trim_start();
            let trimmed_line = trimmed_line
                .strip_prefix("= note: ")
                .unwrap_or(trimmed_line);
            if trimmed_line.starts_with("the full type name has been written to")
                || trimmed_line.starts_with("the full name for the type has been written to")
            {
                return None;
            }
        }

        if self.normalization >= AndOthersVerbose {
            let trim_start = line.trim_start();
            if trim_start.starts_with("= help: the following types implement trait ")
                || trim_start.starts_with("= help: the following other types implement trait ")
            {
                self.other_types = Some(0);
            } else if let Some(count_other_types) = &mut self.other_types {
                if indent >= 12 && trim_start != "and $N others" {
                    *count_other_types += 1;
                    if *count_other_types == 9 {
                        if let Some(next) = self.all_lines.get(index + 1) {
                            let next_trim_start = next.trim_start();
                            let next_indent = next.len() - next_trim_start.len();
                            if indent == next_indent {
                                line.replace_range(indent - 2.., "and $N others");
                            }
                        }
                    } else if *count_other_types > 9 {
                        return None;
                    }
                } else {
                    self.other_types = None;
                }
            }
        }

        line = line.replace(self.context.krate, "$CRATE");
        line = replace_case_insensitive(&line, &self.context.source_dir.to_string_lossy(), "$DIR/");
        line = replace_case_insensitive(
            &line,
            &self.context.workspace.to_string_lossy(),
            "$WORKSPACE/",
        );

        Some(line)
    }
}

fn is_ascii_lowercase_hex(s: &str) -> bool {
    s.bytes().all(|b| matches!(b, b'0'..=b'9' | b'a'..=b'f'))
}

// "10 | T: Send,"  ->  "   | T: Send,"
fn hide_leading_numbers(line: &mut String) {
    let n = line.bytes().take_while(u8::is_ascii_digit).count();
    for i in 0..n {
        line.replace_range(i..i + 1, " ");
    }
}

// "main.rs:22:29"  ->  "main.rs"
fn hide_trailing_numbers(line: &mut String) {
    for _ in 0..2 {
        let digits = line.bytes().rev().take_while(u8::is_ascii_digit).count();
        if digits == 0 || !line[..line.len() - digits].ends_with(':') {
            return;
        }
        line.truncate(line.len() - digits - 1);
    }
}

fn replace_case_insensitive(line: &str, pattern: &str, replacement: &str) -> String {
    let line_lower = line.to_ascii_lowercase().replace('\\', "/");
    let pattern_lower = pattern.to_ascii_lowercase().replace('\\', "/");
    let mut replaced = String::with_capacity(line.len());

    let line_lower = line_lower.as_str();
    let mut split = line_lower.split(&pattern_lower);
    let mut pos = 0;
    let mut insert_replacement = false;
    while let Some(keep) = split.next() {
        if insert_replacement {
            replaced.push_str(replacement);
            pos += pattern.len();
        }
        let mut keep = &line[pos..pos + keep.len()];
        if insert_replacement {
            let end_of_maybe_path = keep.find(&[' ', ':'][..]).unwrap_or(keep.len());
            replaced.push_str(&keep[..end_of_maybe_path].replace('\\', "/"));
            pos += end_of_maybe_path;
            keep = &keep[end_of_maybe_path..];
        }
        replaced.push_str(keep);
        pos += keep.len();
        insert_replacement = true;
        if replaced.ends_with(|ch: char| ch.is_ascii_alphanumeric()) {
            if let Some(ch) = line[pos..].chars().next() {
                replaced.push(ch);
                pos += ch.len_utf8();
                split = line_lower[pos..].split(&pattern_lower);
                insert_replacement = false;
            }
        }
    }

    replaced
}

#[derive(PartialEq)]
enum IndentedLineKind {
    // `error`
    // `warning`
    Heading,

    // Contains max number of spaces that can be cut based on this line.
    // `   --> foo` = 2
    // `    | foo` = 3
    // `   ::: foo` = 2
    // `10  | foo` = 1
    Code(usize),

    // `note:`
    // `...`
    Note,

    // Contains number of leading spaces.
    Other(usize),
}

fn unindent(diag: String, normalization: Normalization) -> String {
    if normalization < Unindent {
        return diag;
    }

    let mut normalized = String::new();
    let mut lines = diag.lines();

    while let Some(line) = lines.next() {
        normalized.push_str(line);
        normalized.push('\n');

        if indented_line_kind(line, &mut false, normalization) != IndentedLineKind::Heading {
            continue;
        }

        let mut ahead = lines.clone();
        let Some(next_line) = ahead.next() else {
            continue;
        };

        if let IndentedLineKind::Code(indent) =
            indented_line_kind(next_line, &mut false, normalization)
        {
            if next_line[indent + 1..].starts_with("--> ") {
                let mut lines_in_block = 1;
                let mut least_indent = indent;
                let mut previous_line_is_note = false;
                while let Some(line) = ahead.next() {
                    match indented_line_kind(line, &mut previous_line_is_note, normalization) {
                        IndentedLineKind::Heading => break,
                        IndentedLineKind::Code(indent) => {
                            lines_in_block += 1;
                            least_indent = cmp::min(least_indent, indent);
                        }
                        IndentedLineKind::Note => lines_in_block += 1,
                        IndentedLineKind::Other(spaces) => {
                            if spaces > 10 {
                                lines_in_block += 1;
                            } else {
                                break;
                            }
                        }
                    }
                }
                previous_line_is_note = false;
                for _ in 0..lines_in_block {
                    let line = lines.next().unwrap();
                    if let IndentedLineKind::Code(_) | IndentedLineKind::Other(_) =
                        indented_line_kind(line, &mut previous_line_is_note, normalization)
                    {
                        let space = line.find(' ').unwrap();
                        normalized.push_str(&line[..space]);
                        normalized.push_str(&line[space + least_indent..]);
                    } else {
                        normalized.push_str(line);
                    }
                    normalized.push('\n');
                }
            }
        }
    }

    normalized
}

fn indented_line_kind(
    line: &str,
    previous_line_is_note: &mut bool,
    normalization: Normalization,
) -> IndentedLineKind {
    let previous_line_was_note = mem::replace(previous_line_is_note, false);

    if let Some(heading_len) = if line.starts_with("error") {
        Some("error".len())
    } else if line.starts_with("warning") {
        Some("warning".len())
    } else {
        None
    } {
        if line[heading_len..].starts_with(&[':', '['][..]) {
            return IndentedLineKind::Heading;
        }
    }

    if line.starts_with("note:")
        || line == "..."
        || normalization >= UnindentAfterHelp && line.starts_with("help:")
        || normalization >= UnindentMultilineNote
            && previous_line_was_note
            && line.starts_with("      ")
    {
        *previous_line_is_note = true;
        return IndentedLineKind::Note;
    }

    let is_space = |b: &u8| *b == b' ';
    if let Some(rest) = line.strip_prefix("... ") {
        let spaces = rest.bytes().take_while(is_space).count();
        return IndentedLineKind::Code(spaces);
    }

    let digits = line.bytes().take_while(u8::is_ascii_digit).count();
    let spaces = line[digits..].bytes().take_while(|b| *b == b' ').count();
    let rest = &line[digits + spaces..];
    if spaces > 0
        && (rest == "|"
            || rest.starts_with("| ")
            || digits == 0
                && (rest.starts_with("--> ") || rest.starts_with("::: ") || rest.starts_with("= ")))
    {
        return IndentedLineKind::Code(spaces - 1);
    }

    IndentedLineKind::Other(if digits == 0 { spaces } else { 0 })
}
