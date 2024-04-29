use std::{fs, path::Path};

pub fn get_files_recursive<P: AsRef<Path>>(
    path: P,
) -> impl Iterator<Item = std::io::Result<fs::DirEntry>> {
    fs::read_dir(path)
        .expect("Directory not found")
        .flat_map(|res| {
            let dir_entry = res.expect("Error reading directory");
            if dir_entry.path().is_dir() {
                Box::new(get_files_recursive(dir_entry.path())) as Box<dyn Iterator<Item = _>>
            } else {
                Box::new(std::iter::once(Ok(dir_entry))) as Box<dyn Iterator<Item = _>>
            }
        })
}
