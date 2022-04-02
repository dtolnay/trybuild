use std::fs;
use std::io::Result;
use std::path::Path;

pub fn write_if_needed<P, C>(path: P, contents: C) -> Result<()>
where
    P: AsRef<Path>,
    C: AsRef<[u8]>,
{
    let path = path.as_ref();
    let contents = contents.as_ref();
    if let Ok(existing_contents) = fs::read(path) {
        if existing_contents == contents {
            return Ok(());
        }
    }
    fs::write(path, contents)
}
