//! 🎨 Test content generators for various scenarios

/// 🌍 Unicode test content for boundary and encoding tests
pub mod unicode {
    /// Complex Unicode content with various character types
    pub fn complex_multiline() -> String {
        [
            "Line 1: Basic ASCII content for padding",
            "Line 2: Unicode mix café résumé naïve 🚀 русский текст 中文字符",
            "Line 3: Emoji combinations 👨‍👩‍👧‍👦 🏳️‍🌈 🏳️‍⚧️ with zero-width joiners",
            "Line 4: Mathematical symbols ∑∫∂∆∇√∞ and arrows ↑↓←→↔",
            "Line 5: More padding to test chunk boundary behavior",
            "Line 6: Final Unicode test with zalgo t̴̹̅e̵̞̓x̶̜̌t̸̰̿ and combining chars áéíóú",
        ].join("\n")
    }

    /// Short Unicode content for simple tests
    pub fn simple() -> &'static str {
        "Hello 🌍 Мир नमस्ते 世界"
    }

    /// Content designed to test specific character boundaries
    pub fn boundary_test() -> String {
        format!("{}\n{}\n{}\n", 
            "测试中文🚀 with émojis and spëcial chars",
            "Zalgo text t̴̹̅e̵̞̓x̶̜̌t̸̰̿ combining chars",
            "Mixed: ASCII + русский + 中文 + 🎉 + mathematical ∑∫∂"
        )
    }

    /// Large Unicode content for performance testing
    pub fn large_repeating(repeat: usize) -> String {
        complex_multiline().repeat(repeat)
    }

    /// File names with Unicode characters
    pub fn unicode_filename() -> &'static str {
        "тест_файл_🌟.txt"
    }

    /// Commit messages with Unicode
    pub fn commit_message() -> &'static str {
        "Добавить файл 📁 with मिश्रित भाषा"
    }
}

/// 📄 Standard file content generators
pub mod files {
    /// Simple text file content
    pub fn simple_text() -> &'static str {
        "Hello, World!\nThis is a test file.\n"
    }

    /// JSON content for testing structured data
    pub fn json_content() -> &'static str {
        r#"{
  "key": "value",
  "array": [1, 2, 3],
  "nested": {
    "unicode": "🚀 test"
  }
}"#
    }

    /// Code file content
    pub fn rust_code() -> &'static str {
        r#"//! Test Rust code
fn main() {
    println!("Hello, 🦀!");
}
"#
    }

    /// Binary-like content (but still valid UTF-8)
    pub fn binary_like() -> String {
        (0..256).map(|i| {
            match i {
                0..=31 | 127 => ' ', // Replace control chars with space
                _ => i as u8 as char,
            }
        }).collect()
    }

    /// Large file content
    pub fn large_text(lines: usize) -> String {
        (0..lines)
            .map(|i| format!("Line {}: Some content with number {}", i, i * 2))
            .collect::<Vec<_>>()
            .join("\n")
    }
}

/// 📁 Directory structure generators
pub mod structures {
    use std::collections::HashMap;

    /// Simple project structure
    pub fn simple_project() -> HashMap<&'static str, &'static str> {
        [
            ("src/main.rs", super::files::rust_code()),
            ("README.md", "# Test Project\n"),
            ("Cargo.toml", "[package]\nname = \"test\"\nversion = \"0.1.0\"\n"),
        ].into_iter().collect()
    }

    /// Nested directory structure
    pub fn nested_structure() -> HashMap<&'static str, &'static str> {
        [
            ("level1/level2/level3/deep.txt", "Deep nested file"),
            ("level1/file1.txt", "Level 1 file"),
            ("level1/level2/file2.txt", "Level 2 file"),
            ("root.txt", "Root level file"),
        ].into_iter().collect()
    }

    /// Mixed content types
    pub fn mixed_content() -> HashMap<&'static str, &'static str> {
        [
            ("text.txt", super::files::simple_text()),
            ("data.json", super::files::json_content()),
            ("unicode.txt", super::unicode::simple()),
            ("code.rs", super::files::rust_code()),
        ].into_iter().collect()
    }
}

/// 🎯 Content for specific tool testing scenarios
pub mod scenarios {
    use super::*;

    /// Content for read_file edge cases
    pub fn read_file_test_content() -> String {
        unicode::complex_multiline()
    }

    /// Content for write_file replacement testing
    pub fn write_file_original() -> String {
        [
            "Line 1: Original content",
            "Line 2: To be replaced",
            "Line 3: Also to be replaced", 
            "Line 4: Will remain",
            "Line 5: Final line",
        ].join("\n")
    }

    pub fn write_file_replacement() -> &'static str {
        "NEW: Replacement content with 🚀 unicode"
    }

    /// Content for replace tool testing
    pub fn replace_test_content() -> &'static str {
        r#"function test() {
    console.log("old value");
    return "old result";
}

const OLD_CONSTANT = "old";
"#
    }

    /// Git test repository content
    pub fn git_test_files() -> Vec<(&'static str, &'static str)> {
        vec![
            ("README.md", "# Test Repository\n"),
            ("src/main.rs", files::rust_code()),
            ("test.txt", unicode::simple()),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unicode_content_generation() {
        let content = unicode::complex_multiline();
        assert!(content.contains("🚀"));
        assert!(content.contains("русский"));
        assert!(content.contains("中文字符"));
        assert!(content.contains("t̴̹̅e̵̞̓x̶̜̌t̸̰̿"));
    }

    #[test]
    fn test_file_content_generation() {
        let json = files::json_content();
        assert!(json.contains("{"));
        assert!(json.contains("🚀"));
        
        let rust = files::rust_code();
        assert!(rust.contains("fn main"));
        assert!(rust.contains("🦀"));
    }

    #[test]
    fn test_structure_generation() {
        let project = structures::simple_project();
        assert!(project.contains_key("src/main.rs"));
        assert!(project.contains_key("Cargo.toml"));
    }
}
