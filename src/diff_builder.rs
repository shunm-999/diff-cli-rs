use crate::file::File;

pub(crate) struct DiffBuilder {}

impl DiffBuilder {
    pub(crate) fn build(source_file: File, target_file: File) -> String {
        let mut result = String::new();

        result.push_str(&format!("--- {}\n", source_file.path));
        result.push_str(&format!("--- {}\n", target_file.path));
        result
    }
}
