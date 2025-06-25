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

const NO_NEWLINE_AT_END_OF_FILE: &str = "\\ No newline at end of file\n";

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
                        result.push_str(NO_NEWLINE_AT_END_OF_FILE);
                    }
                }
                EditTag::Insert { new } => {
                    result.push_str(&format!("+{}\n", new.text));

                    if !new_has_new_line && new.number == new_change_range.end() {
                        result.push_str(NO_NEWLINE_AT_END_OF_FILE);
                    }
                }
                EditTag::Equal { old, new: _new } => {
                    result.push_str(&format!(" {}\n", old.text));

                    if !old_has_new_line && old.number == old_change_range.end() {
                        result.push_str(NO_NEWLINE_AT_END_OF_FILE);
                    }
                    if !new_has_new_line && old.number == new_change_range.end() {
                        result.push_str(NO_NEWLINE_AT_END_OF_FILE);
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::file::File;

    #[test]
    fn test_change_range_methods() {
        let range = ChangeRange { start: 5, count: 3 };
        assert_eq!(range.start(), 5);
        assert_eq!(range.end(), 7);
        assert_eq!(range.count(), 3);
    }

    #[test]
    fn test_calculate_change_range_impl_empty() {
        let mut empty_vec = vec![];
        let range = DiffBuilder::calculate_change_range_impl(&mut empty_vec);
        assert_eq!(range.start, 0);
        assert_eq!(range.count, 0);
    }

    #[test]
    fn test_calculate_change_range_impl_single() {
        let mut single_vec = vec![10];
        let range = DiffBuilder::calculate_change_range_impl(&mut single_vec);
        assert_eq!(range.start, 10);
        assert_eq!(range.count, 1);
    }

    #[test]
    fn test_calculate_change_range_impl_multiple() {
        let mut multiple_vec = vec![5, 6, 7, 10, 11];
        let range = DiffBuilder::calculate_change_range_impl(&mut multiple_vec);
        assert_eq!(range.start, 5);
        assert_eq!(range.count, 7); // 5 to 10 inclusive
    }

    #[test]
    fn test_push_header() {
        let source_file = File {
            path: "source.txt".to_string(),
            content: "old content".to_string(),
        };
        let target_file = File {
            path: "target.txt".to_string(),
            content: "new content".to_string(),
        };

        let mut result = String::new();
        DiffBuilder::push_header(&mut result, &source_file, &target_file);

        assert_eq!(result, "--- source.txt\n+++ target.txt\n");
    }

    #[test]
    fn test_push_change_range() {
        let old_range = ChangeRange { start: 1, count: 3 };
        let new_range = ChangeRange { start: 1, count: 4 };

        let mut result = String::new();
        DiffBuilder::push_change_range(&mut result, &old_range, &new_range);

        assert_eq!(result, "@@ -1,3 +1,4 @@\n");
    }

    #[test]
    fn test_build_identical_files() {
        let source_file = File {
            path: "test1.txt".to_string(),
            content: "line1\nline2\nline3\n".to_string(),
        };
        let target_file = File {
            path: "test2.txt".to_string(),
            content: "line1\nline2\nline3\n".to_string(),
        };

        let result = DiffBuilder::build(source_file, target_file);

        assert!(result.contains("--- test1.txt"));
        assert!(result.contains("+++ test2.txt"));
        assert!(result.contains("@@ -1,4 +1,4 @@"));
        assert!(result.contains(" line1"));
        assert!(result.contains(" line2"));
        assert!(result.contains(" line3"));
    }

    #[test]
    fn test_build_with_deletions() {
        let source_file = File {
            path: "source.txt".to_string(),
            content: "line1\nline2\nline3\nline4\n".to_string(),
        };
        let target_file = File {
            path: "target.txt".to_string(),
            content: "line1\nline3\n".to_string(),
        };

        let result = DiffBuilder::build(source_file, target_file);

        assert!(result.contains("-line2"));
        assert!(result.contains("-line4"));
        assert!(result.contains(" line1"));
        assert!(result.contains(" line3"));
    }

    #[test]
    fn test_build_with_insertions() {
        let source_file = File {
            path: "source.txt".to_string(),
            content: "line1\nline3\n".to_string(),
        };
        let target_file = File {
            path: "target.txt".to_string(),
            content: "line1\nline2\nline3\nline4\n".to_string(),
        };

        let result = DiffBuilder::build(source_file, target_file);

        assert!(result.contains("+line2"));
        assert!(result.contains("+line4"));
        assert!(result.contains(" line1"));
        assert!(result.contains(" line3"));
    }

    #[test]
    fn test_build_with_modifications() {
        let source_file = File {
            path: "source.txt".to_string(),
            content: "line1\nold_line2\nline3\n".to_string(),
        };
        let target_file = File {
            path: "target.txt".to_string(),
            content: "line1\nnew_line2\nline3\n".to_string(),
        };

        let result = DiffBuilder::build(source_file, target_file);

        assert!(result.contains("-old_line2"));
        assert!(result.contains("+new_line2"));
        assert!(result.contains(" line1"));
        assert!(result.contains(" line3"));
    }

    #[test]
    fn test_build_without_trailing_newline() {
        let source_file = File {
            path: "source.txt".to_string(),
            content: "line1\nline2".to_string(),
        };
        let target_file = File {
            path: "target.txt".to_string(),
            content: "line1\nline2\n".to_string(),
        };

        let result = DiffBuilder::build(source_file, target_file);

        assert!(result.contains("\\ No newline at end of file"));
    }

    #[test]
    fn test_build_empty_files() {
        let source_file = File {
            path: "empty1.txt".to_string(),
            content: "".to_string(),
        };
        let target_file = File {
            path: "empty2.txt".to_string(),
            content: "".to_string(),
        };

        let result = DiffBuilder::build(source_file, target_file);

        assert!(result.contains("--- empty1.txt"));
        assert!(result.contains("+++ empty2.txt"));
        // Empty files should have minimal diff output
    }

    #[test]
    fn test_build_single_line_changes() {
        let source_file = File {
            path: "single.txt".to_string(),
            content: "old".to_string(),
        };
        let target_file = File {
            path: "single.txt".to_string(),
            content: "new".to_string(),
        };

        let result = DiffBuilder::build(source_file, target_file);

        assert!(result.contains("-old"));
        assert!(result.contains("+new"));
    }

    #[test]
    fn test_calculate_change_range_with_mixed_edits() {
        // This test would require access to the TextDiff structure
        // For now, we test the public interface through build()
        let source_file = File {
            path: "test.txt".to_string(),
            content: "a\nb\nc\n".to_string(),
        };
        let target_file = File {
            path: "test.txt".to_string(),
            content: "a\nx\nc\n".to_string(),
        };

        let result = DiffBuilder::build(source_file, target_file);

        // Should contain change range information
        assert!(result.contains("@@"));
        // Should show the change
        assert!(result.contains("-b"));
        assert!(result.contains("+x"));
    }
}
