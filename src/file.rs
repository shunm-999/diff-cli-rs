use std::fs;

pub(crate) struct File {
    pub(crate) path: String,
    pub(crate) content: String,
}

pub(crate) struct FileReader {}

impl FileReader {
    pub(crate) fn read(path: String) -> std::io::Result<File> {
        let content = fs::read_to_string(&path)?;
        Ok(File { path, content })
    }
}
