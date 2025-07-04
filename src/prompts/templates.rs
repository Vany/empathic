use serde_json::{Value, json};

/// 💬 MCP Prompts for Editor - Pre-defined prompt templates for common development tasks
pub fn get_prompts_schema() -> Value {
    json!([
        {
            "name": "code_review",
            "description": "🔍 Comprehensive code review with security and performance analysis",
            "arguments": [
                {
                    "name": "file_path",
                    "description": "Path to the file to review",
                    "required": true
                },
                {
                    "name": "focus_areas",
                    "description": "Specific areas to focus on (security, performance, style, logic)",
                    "required": false
                }
            ]
        },
        {
            "name": "rust_optimization",
            "description": "🦀 Rust-specific optimization suggestions and idiomatic patterns",
            "arguments": [
                {
                    "name": "code_snippet",
                    "description": "Rust code to optimize",
                    "required": true
                },
                {
                    "name": "optimization_goal",
                    "description": "Target: performance, memory, readability, or safety",
                    "required": false
                }
            ]
        },
        {
            "name": "debug_assistance",
            "description": "🐛 Debug analysis with error investigation and fix suggestions",
            "arguments": [
                {
                    "name": "error_message",
                    "description": "Error message or compilation error",
                    "required": true
                },
                {
                    "name": "context",
                    "description": "Additional context about what you were trying to do",
                    "required": false
                }
            ]
        },
        {
            "name": "architecture_design",
            "description": "🏗️ Software architecture analysis and design recommendations",
            "arguments": [
                {
                    "name": "project_description",
                    "description": "Description of the project or component",
                    "required": true
                },
                {
                    "name": "constraints",
                    "description": "Technical constraints or requirements",
                    "required": false
                }
            ]
        },
        {
            "name": "rag_query_optimization",
            "description": "🧠 Optimize RAG queries for better semantic search results",
            "arguments": [
                {
                    "name": "query",
                    "description": "Current search query",
                    "required": true
                },
                {
                    "name": "domain",
                    "description": "Domain or topic area for context",
                    "required": false
                }
            ]
        },
        {
            "name": "documentation_generator",
            "description": "📚 Generate comprehensive documentation from code",
            "arguments": [
                {
                    "name": "code_file",
                    "description": "Path to code file to document",
                    "required": true
                },
                {
                    "name": "doc_type",
                    "description": "Type: API, README, inline, or architectural",
                    "required": false
                }
            ]
        },
        {
            "name": "test_strategy",
            "description": "🧪 Test planning and strategy recommendations",
            "arguments": [
                {
                    "name": "component",
                    "description": "Component or module to test",
                    "required": true
                },
                {
                    "name": "test_types",
                    "description": "Types: unit, integration, performance, or security",
                    "required": false
                }
            ]
        },
        {
            "name": "refactoring_plan",
            "description": "♻️ Systematic refactoring strategy with risk assessment",
            "arguments": [
                {
                    "name": "target_code",
                    "description": "Code section to refactor",
                    "required": true
                },
                {
                    "name": "goals",
                    "description": "Refactoring goals: maintainability, performance, or clarity",
                    "required": false
                }
            ]
        },
        {
            "name": "security_audit",
            "description": "🛡️ Security vulnerability assessment and mitigation strategies",
            "arguments": [
                {
                    "name": "scope",
                    "description": "Security audit scope: file, module, or entire project",
                    "required": true
                },
                {
                    "name": "threat_model",
                    "description": "Specific threats to consider",
                    "required": false
                }
            ]
        }
    ])
}

/// Get a specific prompt by name with populated content
pub fn get_prompt(name: &str, arguments: &Value) -> Option<Value> {
    match name {
        "code_review" => Some(build_code_review_prompt(arguments)),
        "rust_optimization" => Some(build_rust_optimization_prompt(arguments)),
        "debug_assistance" => Some(build_debug_assistance_prompt(arguments)),
        "architecture_design" => Some(build_architecture_design_prompt(arguments)),
        "rag_query_optimization" => Some(build_rag_query_optimization_prompt(arguments)),
        "documentation_generator" => Some(build_documentation_generator_prompt(arguments)),
        "test_strategy" => Some(build_test_strategy_prompt(arguments)),
        "refactoring_plan" => Some(build_refactoring_plan_prompt(arguments)),
        "security_audit" => Some(build_security_audit_prompt(arguments)),
        _ => None,
    }
}

fn build_code_review_prompt(args: &Value) -> Value {
    let file_path = args.get("file_path").and_then(|v| v.as_str()).unwrap_or("");
    let focus_areas = args
        .get("focus_areas")
        .and_then(|v| v.as_str())
        .unwrap_or("all areas");

    json!({
        "description": "Comprehensive code review",
        "messages": [
            {
                "role": "user",
                "content": {
                    "type": "text",
                    "text": format!(
                        "🔍 **Code Review Request**\n\n\
                        Please perform a comprehensive code review of: `{}`\n\n\
                        **Focus Areas:** {}\n\n\
                        **Review Criteria:**\n\
                        - 🛡️ Security vulnerabilities and best practices\n\
                        - ⚡ Performance implications and optimizations\n\
                        - 📝 Code clarity and maintainability\n\
                        - 🦀 Language-specific idioms and patterns\n\
                        - 🧪 Testability and error handling\n\
                        - 📚 Documentation and comments\n\n\
                        **Deliverables:**\n\
                        1. Summary assessment with severity levels\n\
                        2. Specific issues with line references\n\
                        3. Actionable improvement suggestions\n\
                        4. Positive highlights and well-implemented patterns\n\n\
                        Please use file operations to read the code and provide detailed feedback.",
                        file_path, focus_areas
                    )
                }
            }
        ]
    })
}

fn build_rust_optimization_prompt(args: &Value) -> Value {
    let code_snippet = args
        .get("code_snippet")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let optimization_goal = args
        .get("optimization_goal")
        .and_then(|v| v.as_str())
        .unwrap_or("overall improvement");

    json!({
        "description": "Rust code optimization analysis",
        "messages": [
            {
                "role": "user",
                "content": {
                    "type": "text",
                    "text": format!(
                        "🦀 **Rust Optimization Analysis**\n\n\
                        **Target:** {}\n\n\
                        **Code to Optimize:**\n```rust\n{}\n```\n\n\
                        **Analysis Framework:**\n\
                        - 🚀 **Performance**: Zero-cost abstractions, allocation patterns\n\
                        - 💾 **Memory**: Ownership, borrowing, lifetime optimizations\n\
                        - 📖 **Readability**: Idiomatic Rust patterns, clarity\n\
                        - 🛡️ **Safety**: Memory safety, thread safety, error handling\n\n\
                        **Deliverables:**\n\
                        1. Optimized code with explanations\n\
                        2. Performance impact analysis\n\
                        3. Trade-offs and alternatives\n\
                        4. Benchmark suggestions if applicable\n\n\
                        Focus on modern Rust idioms and zero-cost abstractions.",
                        optimization_goal, code_snippet
                    )
                }
            }
        ]
    })
}

fn build_debug_assistance_prompt(args: &Value) -> Value {
    let error_message = args
        .get("error_message")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let context = args
        .get("context")
        .and_then(|v| v.as_str())
        .unwrap_or("No additional context provided");

    json!({
        "description": "Debug assistance and error resolution",
        "messages": [
            {
                "role": "user",
                "content": {
                    "type": "text",
                    "text": format!(
                        "🐛 **Debug Assistance Request**\n\n\
                        **Error Message:**\n```\n{}\n```\n\n\
                        **Context:**\n{}\n\n\
                        **Debug Strategy:**\n\
                        1. 🔍 **Error Analysis**: Root cause identification\n\
                        2. 🛠️ **Fix Suggestions**: Step-by-step resolution\n\
                        3. 🧪 **Verification**: Testing approach for the fix\n\
                        4. 🚀 **Prevention**: How to avoid similar issues\n\n\
                        **Investigation Tools:**\n\
                        - Use file operations to examine related code\n\
                        - Search for similar patterns in the codebase\n\
                        - Check dependencies and configurations\n\n\
                        Please provide actionable debugging steps and fix recommendations.",
                        error_message, context
                    )
                }
            }
        ]
    })
}

fn build_architecture_design_prompt(args: &Value) -> Value {
    let project_description = args
        .get("project_description")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let constraints = args
        .get("constraints")
        .and_then(|v| v.as_str())
        .unwrap_or("No specific constraints");

    json!({
        "description": "Software architecture design consultation",
        "messages": [
            {
                "role": "user",
                "content": {
                    "type": "text",
                    "text": format!(
                        "🏗️ **Architecture Design Consultation**\n\n\
                        **Project:** {}\n\n\
                        **Constraints:** {}\n\n\
                        **Design Framework:**\n\
                        - 🎯 **Requirements Analysis**: Functional and non-functional\n\
                        - 🏛️ **Architectural Patterns**: Suitable design patterns\n\
                        - 🔧 **Technology Stack**: Language, frameworks, databases\n\
                        - 📊 **Scalability**: Growth and performance considerations\n\
                        - 🛡️ **Security**: Security architecture and threat model\n\
                        - 🧪 **Testing**: Testing strategy and quality assurance\n\n\
                        **Deliverables:**\n\
                        1. High-level architecture diagram (text-based)\n\
                        2. Component breakdown with responsibilities\n\
                        3. Data flow and interface definitions\n\
                        4. Technology recommendations with rationale\n\
                        5. Implementation roadmap with priorities\n\n\
                        Focus on maintainable, scalable, and robust design principles.",
                        project_description, constraints
                    )
                }
            }
        ]
    })
}

fn build_rag_query_optimization_prompt(args: &Value) -> Value {
    let query = args.get("query").and_then(|v| v.as_str()).unwrap_or("");
    let domain = args
        .get("domain")
        .and_then(|v| v.as_str())
        .unwrap_or("general");

    json!({
        "description": "RAG query optimization for better retrieval",
        "messages": [
            {
                "role": "user",
                "content": {
                    "type": "text",
                    "text": format!(
                        "🧠 **RAG Query Optimization**\n\n\
                        **Current Query:** {}\n\
                        **Domain:** {}\n\n\
                        **Optimization Strategy:**\n\
                        - 🎯 **Semantic Enhancement**: Improve query semantics\n\
                        - 🔍 **Keyword Expansion**: Add relevant synonyms and terms\n\
                        - 📊 **Query Structuring**: Optimize for vector similarity\n\
                        - 🎛️ **Parameter Tuning**: Suggest search parameters\n\n\
                        **Analysis Areas:**\n\
                        1. Query intent and information need\n\
                        2. Domain-specific terminology\n\
                        3. Semantic similarity patterns\n\
                        4. Chunk retrieval effectiveness\n\n\
                        **Deliverables:**\n\
                        - Optimized query variations\n\
                        - Search strategy recommendations\n\
                        - RAG parameter suggestions\n\
                        - Expected retrieval improvements\n\n\
                        Use RAG tools to test query effectiveness if helpful.",
                        query, domain
                    )
                }
            }
        ]
    })
}

fn build_documentation_generator_prompt(args: &Value) -> Value {
    let code_file = args.get("code_file").and_then(|v| v.as_str()).unwrap_or("");
    let doc_type = args
        .get("doc_type")
        .and_then(|v| v.as_str())
        .unwrap_or("comprehensive");

    json!({
        "description": "Generate comprehensive documentation",
        "messages": [
            {
                "role": "user",
                "content": {
                    "type": "text",
                    "text": format!(
                        "📚 **Documentation Generation**\n\n\
                        **Target File:** {}\n\
                        **Documentation Type:** {}\n\n\
                        **Documentation Framework:**\n\
                        - 📋 **API Documentation**: Function signatures and usage\n\
                        - 💡 **Examples**: Practical usage examples\n\
                        - 🏗️ **Architecture**: Module structure and relationships\n\
                        - 🚀 **Getting Started**: Quick start guides\n\
                        - ⚠️ **Edge Cases**: Error handling and limitations\n\n\
                        **Output Requirements:**\n\
                        1. Clear, concise descriptions\n\
                        2. Code examples with explanations\n\
                        3. Proper markdown formatting\n\
                        4. Cross-references and links\n\
                        5. Maintainable documentation structure\n\n\
                        Please read the code file and generate appropriate documentation.\n\
                        Include inline doc comments and external documentation as needed.",
                        code_file, doc_type
                    )
                }
            }
        ]
    })
}

fn build_test_strategy_prompt(args: &Value) -> Value {
    let component = args.get("component").and_then(|v| v.as_str()).unwrap_or("");
    let test_types = args
        .get("test_types")
        .and_then(|v| v.as_str())
        .unwrap_or("comprehensive");

    json!({
        "description": "Test strategy development",
        "messages": [
            {
                "role": "user",
                "content": {
                    "type": "text",
                    "text": format!(
                        "🧪 **Test Strategy Development**\n\n\
                        **Component:** {}\n\
                        **Test Types:** {}\n\n\
                        **Testing Framework:**\n\
                        - 🔬 **Unit Tests**: Individual function testing\n\
                        - 🔗 **Integration Tests**: Component interaction testing\n\
                        - ⚡ **Performance Tests**: Load and stress testing\n\
                        - 🛡️ **Security Tests**: Vulnerability and penetration testing\n\
                        - 👤 **User Acceptance**: End-to-end workflow testing\n\n\
                        **Deliverables:**\n\
                        1. Test plan with coverage targets\n\
                        2. Test case specifications\n\
                        3. Testing infrastructure requirements\n\
                        4. Automation strategy\n\
                        5. Quality gates and success criteria\n\n\
                        **Implementation Guidance:**\n\
                        - Rust-specific testing patterns\n\
                        - Mocking and test doubles\n\
                        - Continuous integration setup\n\n\
                        Analyze the component and provide a comprehensive testing strategy.",
                        component, test_types
                    )
                }
            }
        ]
    })
}

fn build_refactoring_plan_prompt(args: &Value) -> Value {
    let target_code = args
        .get("target_code")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let goals = args
        .get("goals")
        .and_then(|v| v.as_str())
        .unwrap_or("general improvement");

    json!({
        "description": "Systematic refactoring strategy",
        "messages": [
            {
                "role": "user",
                "content": {
                    "type": "text",
                    "text": format!(
                        "♻️ **Refactoring Strategy**\n\n\
                        **Target Code:** {}\n\
                        **Goals:** {}\n\n\
                        **Refactoring Process:**\n\
                        - 🔍 **Analysis**: Current state assessment\n\
                        - 🎯 **Objectives**: Clear refactoring goals\n\
                        - 📋 **Plan**: Step-by-step refactoring plan\n\
                        - ⚠️ **Risk Assessment**: Potential breaking changes\n\
                        - 🧪 **Validation**: Testing strategy for changes\n\n\
                        **Key Considerations:**\n\
                        1. Backward compatibility requirements\n\
                        2. Performance impact analysis\n\
                        3. Team coordination and communication\n\
                        4. Incremental delivery approach\n\
                        5. Rollback strategies\n\n\
                        **Deliverables:**\n\
                        - Detailed refactoring roadmap\n\
                        - Risk mitigation strategies\n\
                        - Before/after code comparisons\n\
                        - Testing and validation plan\n\n\
                        Provide a safe, systematic approach to code improvement.",
                        target_code, goals
                    )
                }
            }
        ]
    })
}

fn build_security_audit_prompt(args: &Value) -> Value {
    let scope = args.get("scope").and_then(|v| v.as_str()).unwrap_or("");
    let threat_model = args
        .get("threat_model")
        .and_then(|v| v.as_str())
        .unwrap_or("general security threats");

    json!({
        "description": "Security vulnerability assessment",
        "messages": [
            {
                "role": "user",
                "content": {
                    "type": "text",
                    "text": format!(
                        "🛡️ **Security Audit Request**\n\n\
                        **Audit Scope:** {}\n\
                        **Threat Model:** {}\n\n\
                        **Security Assessment Framework:**\n\
                        - 🔒 **Authentication**: Identity verification\n\
                        - 🚪 **Authorization**: Access control mechanisms\n\
                        - 🛡️ **Input Validation**: Data sanitization and validation\n\
                        - 🔐 **Cryptography**: Encryption and key management\n\
                        - 📊 **Data Protection**: Privacy and data handling\n\
                        - 🌐 **Network Security**: Communication security\n\n\
                        **Vulnerability Categories:**\n\
                        1. OWASP Top 10 compliance\n\
                        2. Memory safety issues (Rust-specific)\n\
                        3. Dependency vulnerabilities\n\
                        4. Configuration security\n\
                        5. Business logic flaws\n\n\
                        **Deliverables:**\n\
                        - Security vulnerability report\n\
                        - Risk assessment with severity levels\n\
                        - Remediation recommendations\n\
                        - Security best practices guide\n\n\
                        Perform thorough security analysis using file operations and search tools.",
                        scope, threat_model
                    )
                }
            }
        ]
    })
}


