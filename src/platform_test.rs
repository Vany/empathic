use crate::common::{send_error, send_response};
use crate::modules::memory::get_memory_store;
use crate::platform::{Platform, ToolCompatibility, get_tts};
use crate::tools::tool_trait::Tool;
use crate::register_tool;
use serde_json::{Value, json};

/// 🧪 Platform compatibility test suite
#[derive(Default)]
pub struct PlatformTestTool;

impl Tool for PlatformTestTool {
    fn name(&self) -> &'static str {
        "platform_test"
    }
    fn description(&self) -> &'static str {
        "Run comprehensive platform compatibility tests"
    }
    fn emoji(&self) -> &'static str {
        "🧪"
    }

    fn schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "test_suite": {
                    "type": "string",
                    "enum": ["all", "platform", "tts", "memory", "security"],
                    "default": "all",
                    "description": "Test suite to run"
                }
            }
        })
    }

    fn execute_impl(&self, id: u64, args: Option<Value>) {
        let test_suite = args
            .as_ref()
            .and_then(|a| a.get("test_suite"))
            .and_then(|t| t.as_str())
            .unwrap_or("all");

        let mut results = TestResults::new();

        match test_suite {
            "all" => {
                results.run_platform_tests();
                results.run_tts_tests();
                results.run_memory_tests();
                results.run_security_tests();
            }
            "platform" => results.run_platform_tests(),
            "tts" => results.run_tts_tests(),
            "memory" => results.run_memory_tests(),
            "security" => results.run_security_tests(),
            _ => {
                send_error(id, -1, "Invalid test suite");
                return;
            }
        }

        let report = results.generate_report();
        send_response(
            id,
            json!({
                "content": [{
                    "type": "text",
                    "text": report
                }]
            }),
        );
    }
}

register_tool!("platform_test", PlatformTestTool);

struct TestResults {
    tests: Vec<TestResult>,
}

struct TestResult {
    name: String,
    passed: bool,
    message: String,
    category: String,
}

impl TestResults {
    fn new() -> Self {
        Self { tests: Vec::new() }
    }

    fn add_test(&mut self, category: &str, name: &str, passed: bool, message: &str) {
        self.tests.push(TestResult {
            name: name.to_string(),
            passed,
            message: message.to_string(),
            category: category.to_string(),
        });
    }

    fn run_platform_tests(&mut self) {
        // Test platform detection
        let platform = Platform::current();
        self.add_test(
            "Platform",
            "Detection",
            true,
            &format!(
                "Detected: {} {}",
                platform.emoji(),
                match platform {
                    Platform::MacOS => "macOS",
                    Platform::Linux => "Linux",
                    Platform::Windows => "Windows",
                    Platform::Unknown => "Unknown",
                }
            ),
        );

        // Test tool compatibility
        let compat = ToolCompatibility::for_current_platform();
        self.add_test(
            "Platform",
            "Git Available",
            compat.supports_git,
            if compat.supports_git {
                "git command found"
            } else {
                "git not available"
            },
        );
        self.add_test(
            "Platform",
            "Cargo Available",
            compat.supports_cargo,
            if compat.supports_cargo {
                "cargo command found"
            } else {
                "cargo not available"
            },
        );
        self.add_test(
            "Platform",
            "Make Available",
            compat.supports_make,
            if compat.supports_make {
                "make command found"
            } else {
                "make not available"
            },
        );
    }

    fn run_tts_tests(&mut self) {
        let tts = get_tts();
        let available = tts.is_available();

        self.add_test(
            "TTS",
            "Availability",
            available,
            if available {
                "TTS system available"
            } else {
                "TTS not available"
            },
        );

        if available {
            match tts.speak("test") {
                Ok(_) => self.add_test("TTS", "Synthesis", true, "TTS synthesis working"),
                Err(e) => self.add_test("TTS", "Synthesis", false, &e),
            }
        }
    }

    fn run_memory_tests(&mut self) {
        let store = get_memory_store();

        // Test basic operations
        let test_key = "test_key_123";
        let test_value = json!({"test": "data", "number": 42});

        match store.store(
            test_key.to_string(),
            test_value.clone(),
            vec!["test".to_string()],
        ) {
            Ok(_) => self.add_test("Memory", "Store", true, "Memory store working"),
            Err(e) => self.add_test("Memory", "Store", false, &e),
        }

        match store.retrieve(test_key) {
            Ok(Some(value)) if value == test_value => {
                self.add_test("Memory", "Retrieve", true, "Memory retrieve working");
            }
            Ok(Some(_)) => self.add_test("Memory", "Retrieve", false, "Retrieved wrong value"),
            Ok(None) => self.add_test("Memory", "Retrieve", false, "Key not found"),
            Err(e) => self.add_test("Memory", "Retrieve", false, &e),
        }

        match store.search("test", true) {
            Ok(results) if !results.is_empty() => {
                self.add_test("Memory", "Search", true, "Memory search working");
            }
            Ok(_) => self.add_test("Memory", "Search", false, "Search returned no results"),
            Err(e) => self.add_test("Memory", "Search", false, &e),
        }

        // Cleanup
        let _ = store.delete(test_key);
    }

    fn run_security_tests(&mut self) {
        use crate::modules::security::get_security_validator;

        let validator = get_security_validator();

        // Test valid path
        match validator.validate_path("test.txt") {
            Ok(_) => self.add_test("Security", "Valid Path", true, "Path validation working"),
            Err(e) => self.add_test("Security", "Valid Path", false, &e.to_string()),
        }

        // Test path traversal prevention
        match validator.validate_path("../../../etc/passwd") {
            Ok(_) => self.add_test(
                "Security",
                "Path Traversal",
                false,
                "Path traversal not blocked",
            ),
            Err(_) => self.add_test("Security", "Path Traversal", true, "Path traversal blocked"),
        }

        // Test blocked extensions
        match validator.validate_path("malware.exe") {
            Ok(_) => self.add_test(
                "Security",
                "Blocked Extension",
                false,
                "Blocked extension allowed",
            ),
            Err(_) => self.add_test(
                "Security",
                "Blocked Extension",
                true,
                "Blocked extension rejected",
            ),
        }
    }

    fn generate_report(&self) -> String {
        let mut report = String::from("🧪 Platform Compatibility Test Results\n");
        report.push_str("==========================================\n\n");

        let total_tests = self.tests.len();
        let passed_tests = self.tests.iter().filter(|t| t.passed).count();
        let failed_tests = total_tests - passed_tests;

        report.push_str(&format!(
            "📊 Summary: {}/{} tests passed ({}%)\n",
            passed_tests,
            total_tests,
            if total_tests > 0 {
                passed_tests * 100 / total_tests
            } else {
                0
            }
        ));
        report.push_str(&format!("✅ Passed: {passed_tests}\n"));
        report.push_str(&format!("❌ Failed: {failed_tests}\n\n"));

        // Group by category
        let mut categories: std::collections::HashMap<String, Vec<&TestResult>> =
            std::collections::HashMap::new();
        for test in &self.tests {
            categories
                .entry(test.category.clone())
                .or_default()
                .push(test);
        }

        for (category, tests) in categories {
            let passed_in_category = tests.iter().filter(|t| t.passed).count();
            let emoji = match category.as_str() {
                "Platform" => "🌐",
                "TTS" => "🔊",
                "Memory" => "💾",
                "Security" => "🛡️",
                _ => "🔧",
            };

            report.push_str(&format!(
                "{} {} Tests ({}/{})\n",
                emoji,
                category,
                passed_in_category,
                tests.len()
            ));
            report.push_str(&format!("{}\n", "=".repeat(category.len() + 15)));

            for test in tests {
                let status = if test.passed { "✅" } else { "❌" };
                report.push_str(&format!("{} {}: {}\n", status, test.name, test.message));
            }
            report.push('\n');
        }

        if failed_tests == 0 {
            report.push_str("🎉 All tests passed! Platform compatibility confirmed.\n");
        } else {
            report.push_str("⚠️ Some tests failed. Check platform-specific requirements.\n");
        }

        report
    }
}
