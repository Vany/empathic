/// 🏭 **UNIFORM** Executor tool creation macro - same behavior for all tools
/// 
/// Creates a tool struct that implements the Tool trait by calling the provided executor function.
/// All command-specific logic is moved into the executor function parameter.
#[macro_export]
macro_rules! create_executor_tool {
    (
        $tool_struct:ident,
        $executor_fn:expr,        // Executor function to call: fn(ExecutorArgs) -> Result<ExecutorResult, String>
        $tool_name:expr,
        $desc:expr,
        $emoji:expr,
        $schema:expr
    ) => {
        #[derive(Default)]
        pub struct $tool_struct;

        impl $crate::tools::tool_trait::Tool for $tool_struct {
            fn name(&self) -> &'static str {
                $tool_name
            }
            
            fn description(&self) -> &'static str {
                $desc
            }
            
            fn emoji(&self) -> &'static str {
                $emoji
            }
            
            fn schema(&self) -> serde_json::Value {
                $schema
            }
            
            fn execute_impl(&self, id: u64, args: Option<serde_json::Value>) {
                use $crate::executor::ExecutorArgs;
                use $crate::common::{send_error, send_response};
                use serde_json::json;
                
                let executor_args: ExecutorArgs = match args {
                    Some(value) => serde_json::from_value(value).unwrap_or_default(),
                    None => ExecutorArgs::default(),
                };
                
                // 🎯 **UNIFORM EXECUTION** - All tools use the same pattern
                match $executor_fn(executor_args) {
                    Ok(exec_result) => {
                        let status_icon = if exec_result.success { "✅" } else { "❌" };
                        let status_text = if exec_result.success { "Success" } else { "Failed" };
                        
                        let formatted_output = format!(
                            "{} {} (Exit code: {})\n🔨 Command: {}\n\n📄 Output:\n{}",
                            status_icon,
                            status_text,
                            exec_result.exit_code,
                            $tool_name,  // Simple tool name as command display
                            exec_result.output
                        );
                        
                        let response = json!({
                            "content": [{
                                "type": "text",
                                "text": formatted_output
                            }]
                        });
                        
                        send_response(id, response);
                    }
                    Err(error) => {
                        send_error(id, -1, &error);
                    }
                }
            }
        }
    };
}

/// 🔧 Tool registration macro - registers tool with the global registry using existing system
#[macro_export]
macro_rules! register_executor_tool {
    ($tool_name:expr, $tool_type:ty) => {
        $crate::register_tool!($tool_name, $tool_type);
    };
}