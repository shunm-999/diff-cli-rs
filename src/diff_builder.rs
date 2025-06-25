use crate::file::File;
use myers::myers::text_diff::{EditTag, TextDiff};

pub(crate) struct DiffBuilder {}

struct ChangeRange {
    start: u64,
    count: u64,
}

impl DiffBuilder {
    pub(crate) fn build(source_file: File, target_file: File) -> String {
        let mut result = String::new();

        let diff = TextDiff::from_lines(&source_file.content, &target_file.content);
        let (old_change_range, new_change_range) = Self::calculate_change_range(&diff);

        Self::push_header(&mut result, &source_file, &target_file);
        Self::push_change_range(&mut result, &old_change_range, &new_change_range);
        Self::push_change(&mut result, &diff);
        result
    }

    fn calculate_change_range(diff: &TextDiff) -> (ChangeRange, ChangeRange) {
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

        let old_change_range = Self::calculate_change_range_impl(&mut old_line_numbers);
        let new_change_range = Self::calculate_change_range_impl(&mut new_line_numbers);

        (old_change_range, new_change_range)
    }

    fn calculate_change_range_impl(line_numbers: &mut Vec<u64>) -> ChangeRange {
        let old_start = line_numbers.iter().min().copied().unwrap_or(0);
        let old_end = line_numbers.iter().max().copied().unwrap_or(0);
        let old_count = if line_numbers.is_empty() {
            0
        } else {
            old_end - old_start + 1
        };
        ChangeRange {
            start: old_start,
            count: old_count,
        }
    }

    fn push_header(result: &mut String, source_file: &File, target_file: &File) {
        result.push_str(&format!("--- {}\n", source_file.path));
        result.push_str(&format!("+++ {}\n", target_file.path));
    }

    fn push_change_range(
        result: &mut String,
        old_change_range: &ChangeRange,
        new_change_range: &ChangeRange,
    ) {
        result.push_str(&format!(
            "@@ -{},{} +{},{} @@\n",
            old_change_range.start,
            old_change_range.count,
            new_change_range.start,
            new_change_range.count
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
