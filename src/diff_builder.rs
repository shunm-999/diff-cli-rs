use crate::file::File;
use myers::myers::text_diff::{EditTag, TextDiff};

pub(crate) struct DiffBuilder {}

impl DiffBuilder {
    pub(crate) fn build(source_file: File, target_file: File) -> String {
        let mut result = String::new();
        let diff = TextDiff::from_lines(&source_file.content, &target_file.content);

        Self::push_header(&mut result, &source_file, &target_file);
        Self::push_change_range(&mut result, &diff);
        Self::push_change(&mut result, &diff);
        result
    }

    fn push_header(result: &mut String, source_file: &File, target_file: &File) {
        result.push_str(&format!("--- {}\n", source_file.path));
        result.push_str(&format!("+++ {}\n", target_file.path));
    }

    fn push_change_range(result: &mut String, diff: &TextDiff) {
        let mut old_line_numbers = vec![];
        let mut new_line_numbers = vec![];

        for edit in &diff.edits {
            match edit {
                EditTag::Delete { old } => {
                    old_line_numbers.push(old.number);
                }
                EditTag::Insert { new } => {
                    new_line_numbers.push(new.number);
                }
                EditTag::Equal { old, new } => {
                    old_line_numbers.push(old.number);
                    new_line_numbers.push(new.number);
                }
            }
        }

        let old_start = old_line_numbers.iter().min().copied().unwrap_or(0);
        let old_end = old_line_numbers.iter().max().copied().unwrap_or(0);
        let old_count = if old_line_numbers.is_empty() {
            0
        } else {
            old_end - old_start + 1
        };

        let new_start = new_line_numbers.iter().min().copied().unwrap_or(0);
        let new_end = new_line_numbers.iter().max().copied().unwrap_or(0);
        let new_count = if new_line_numbers.is_empty() {
            0
        } else {
            new_end - new_start + 1
        };

        result.push_str(&format!(
            "@@ -{},{} +{},{} @@\n",
            old_start, old_count, new_start, new_count
        ));
    }

    fn push_change(result: &mut String, diff: &TextDiff) {
        for edit in &diff.edits {
            match edit {
                EditTag::Delete { old } => {
                    result.push_str(&format!("-{}\n", old.text));
                }
                EditTag::Insert { new } => {
                    result.push_str(&format!("+{}\n", new.text));
                }
                EditTag::Equal { old, new: _new } => {
                    result.push_str(&format!(" {}\n", old.text));
                }
            }
        }
    }
}
