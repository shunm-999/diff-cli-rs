use crate::file::File;
use myers::myers::text_diff::{EditTag, TextDiff};

pub(crate) struct DiffBuilder {}

struct ChangeRange {
    start: u64,
    count: u64,
}

impl ChangeRange {
    fn start(&self) -> u64 {
        self.start
    }
    fn end(&self) -> u64 {
        self.start + self.count - 1
    }
    fn count(&self) -> u64 {
        self.count
    }
}

impl DiffBuilder {
    pub(crate) fn build(source_file: File, target_file: File) -> String {
        let mut result = String::new();

        let diff = TextDiff::from_lines(&source_file.content, &target_file.content);
        let (old_change_range, new_change_range) = Self::calculate_change_range(&diff);

        let old_has_new_line = &source_file.content.ends_with('\n');
        let new_has_new_line = &target_file.content.ends_with('\n');

        Self::push_header(&mut result, &source_file, &target_file);
        Self::push_change_range(&mut result, &old_change_range, &new_change_range);
        Self::push_change(
            &mut result,
            &diff,
            &old_change_range,
            &new_change_range,
            *old_has_new_line,
            *new_has_new_line,
        );
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

    fn push_change(
        result: &mut String,
        diff: &TextDiff,
        old_change_range: &ChangeRange,
        new_change_range: &ChangeRange,
        old_has_new_line: bool,
        new_has_new_line: bool,
    ) {
        for edit in &diff.edits {
            match edit {
                EditTag::Delete { old } => {
                    result.push_str(&format!("-{}\n", old.text));

                    if !old_has_new_line && old.number == old_change_range.end() {
                        result.push_str("\\ No newline at end of file\n");
                    }
                }
                EditTag::Insert { new } => {
                    result.push_str(&format!("+{}\n", new.text));

                    if !new_has_new_line && new.number == new_change_range.end() {
                        result.push_str("\\ No newline at end of file\n");
                    }
                }
                EditTag::Equal { old, new: _new } => {
                    result.push_str(&format!(" {}\n", old.text));

                    if !old_has_new_line && old.number == old_change_range.end() {
                        result.push_str("\\ No newline at end of file\n");
                    }
                    if !new_has_new_line && old.number == new_change_range.end() {
                        result.push_str("\\ No newline at end of file\n");
                    }
                }
            }
        }
    }
}
