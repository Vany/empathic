use crate::{
    common::{send_error, send_response},
    rag::{
        rag_stack::{get_rag_stack, get_rag_stack_status, is_rag_stack_healthy, shutdown_rag_stack},
    },
};
use super::tool_trait::Tool;
use crate::register_tool;
use serde::Deserialize;
use serde_json::{Value, json};

/// 🏥 RAG Stack Health Tool
/// Checks the health status of all RAG services (auto-starts if needed)
#[derive(Default)]
pub struct RagHealthTool;

impl Tool for RagHealthTool {
    fn name(&self) -> &'static str {
        "rag_health"
    }

    fn description(&self) -> &'static str {
        "Check the health status of all RAG services including detailed diagnostics. This tool provides comprehensive health monitoring: 1) Elasticsearch cluster health (green/yellow/red status), 2) Embeddings service health and model loading status, 3) API endpoint availability, 4) Overall stack readiness for operations. Returns detailed status information including service URLs, response times, and any error conditions. Use this tool to: verify services are running after startup, diagnose issues with RAG operations, monitor service health during development, troubleshoot connectivity problems. The tool performs actual HTTP health checks to validate service availability."
    }

    fn emoji(&self) -> &'static str {
        "🏥"
    }

    fn schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {},
            "additionalProperties": false,
            "description": "Check RAG services health status - no parameters required"
        })
    }

    fn execute_impl(&self, id: u64, _args: Option<Value>) {
        tokio::spawn(async move {
            // Auto-start stack if needed
            let stack = get_rag_stack();
            let stack_guard = stack.lock().await;
            if let Err(e) = stack_guard.ensure_running().await {
                send_error(
                    id,
                    -1,
                    &format!("❌ Failed to ensure RAG stack is running: {e}"),
                );
                return;
            }
            drop(stack_guard);

            let is_healthy = is_rag_stack_healthy().await;
            let status = get_rag_stack_status().await;

            let health_emoji = if is_healthy { "✅" } else { "❌" };
            let overall_status = if is_healthy { "HEALTHY" } else { "UNHEALTHY" };

            let result = json!({
                "content": [{
                    "type": "text",
                    "text": format!(
                        "🏥 RAG Stack Health Check\n\n\
                        {} Overall Status: {} (Auto-managed)\n\n\
                        📊 Elasticsearch:\n\
                        - Status: {}\n\
                        - URL: http://localhost:9200\n\
                        - Health endpoint: /_cluster/health\n\n\
                        🧠 Embeddings Service:\n\
                        - Status: {}\n\
                        - URL: http://localhost:8000\n\
                        - Health endpoint: /health\n\
                        - API docs: /docs\n\n\
                        📋 Detailed Status:\n{}\n\n\
                        💡 Notes:\n\
                        - Stack auto-starts on first RAG operation\n\
                        - Shuts down automatically when editor exits\n\
                        - If unhealthy, try: rag_restart\n\
                        - Check logs with: rag_logs",
                        health_emoji,
                        overall_status,
                        if status["elasticsearch"]["healthy"].as_bool().unwrap_or(false) { "✅ Healthy" } else { "❌ Unhealthy" },
                        if status["embeddings"]["healthy"].as_bool().unwrap_or(false) { "✅ Healthy" } else { "❌ Unhealthy" },
                        serde_json::to_string_pretty(&status).unwrap_or_else(|_| "Status unavailable".to_string())
                    )
                }]
            });
            send_response(id, result);
        });
    }
}

/// 🔄 RAG Stack Restart Tool
/// Restarts specific services or the entire stack
#[derive(Default)]
pub struct RagRestartTool;

#[derive(Deserialize, Default)]
pub struct RagRestartArgs {
    service: Option<String>,
}

impl Tool for RagRestartTool {
    fn name(&self) -> &'static str {
        "rag_restart"
    }

    fn description(&self) -> &'static str {
        "Restart RAG services to recover from errors or apply configuration changes. Can restart individual services or the entire stack: 1) Individual service restart: specify 'elasticsearch' or 'embeddings' to restart just that service while keeping others running, 2) Full stack restart: omit service parameter to restart everything. This is useful for: recovering from service failures, applying configuration changes, clearing memory issues, resolving connectivity problems. Individual restarts are faster and less disruptive. Full restart ensures clean state but takes longer. Data in persistent volumes is preserved during restarts."
    }

    fn emoji(&self) -> &'static str {
        "🔄"
    }

    fn schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "service": {
                    "type": "string",
                    "description": "Optional: specific service to restart ('elasticsearch' or 'embeddings'). If not specified, restarts entire stack.",
                    "enum": ["elasticsearch", "embeddings"]
                }
            },
            "additionalProperties": false,
            "description": "Restart RAG services - optionally specify individual service"
        })
    }

    fn execute_impl(&self, id: u64, args: Option<Value>) {
        tokio::spawn(async move {
            let args: RagRestartArgs = args
                .and_then(|v| serde_json::from_value(v).ok())
                .unwrap_or_default();

            let stack = get_rag_stack();
            let stack_guard = stack.lock().await;

            // Ensure stack is running first
            if let Err(e) = stack_guard.ensure_running().await {
                send_error(
                    id,
                    -1,
                    &format!("❌ Failed to ensure RAG stack is running: {e}"),
                );
                return;
            }

            match args.service {
                Some(service) => match stack_guard.restart_service(&service).await {
                    Ok(_) => {
                        let result = json!({
                            "content": [{
                                "type": "text",
                                "text": format!(
                                    "🔄 Service '{}' Restarted Successfully!\n\n\
                                    ✅ Service is restarting\n\
                                    ⏳ Health checks will run automatically\n\
                                    🔍 Use rag_health to verify status\n\n\
                                    💡 Service endpoints:\n{}",
                                    service,
                                    if service == "elasticsearch" {
                                        "📊 Elasticsearch: http://localhost:9200"
                                    } else {
                                        "🧠 Embeddings: http://localhost:8000"
                                    }
                                )
                            }]
                        });
                        send_response(id, result);
                    }
                    Err(e) => send_error(
                        id,
                        -1,
                        &format!("❌ Failed to restart service '{service}': {e}"),
                    ),
                },
                None => {
                    // Full stack restart - shutdown and let auto-start handle restart
                    match stack_guard.shutdown() {
                        Ok(_) => {
                            drop(stack_guard); // Release lock for restart
                            let stack_guard = stack.lock().await;
                            match stack_guard.ensure_running().await {
                                Ok(_) => {
                                    let result = json!({
                                        "content": [{
                                            "type": "text",
                                            "text": "🔄 RAG Stack Restarted Successfully!\n\n\
                                                    ✅ All services restarted (auto-managed)\n\
                                                    📊 Elasticsearch: http://localhost:9200\n\
                                                    🧠 Embeddings: http://localhost:8000\n\
                                                    🔍 Health checks completed\n\n\
                                                    💡 Stack is ready for operations!"
                                        }]
                                    });
                                    send_response(id, result);
                                }
                                Err(e) => {
                                    send_error(id, -1, &format!("❌ Failed to restart stack: {e}"))
                                }
                            }
                        }
                        Err(e) => send_error(
                            id,
                            -1,
                            &format!("❌ Failed to shutdown stack for restart: {e}"),
                        ),
                    }
                }
            }
        });
    }
}

/// 📋 RAG Stack Logs Tool
/// Retrieves logs from RAG services for debugging
#[derive(Default)]
pub struct RagLogsTool;

#[derive(Deserialize, Default)]
pub struct RagLogsArgs {
    service: Option<String>,
    lines: Option<u32>,
}

impl Tool for RagLogsTool {
    fn name(&self) -> &'static str {
        "rag_logs"
    }

    fn description(&self) -> &'static str {
        "Retrieve logs from RAG services for debugging and monitoring. This tool helps diagnose issues by showing recent log entries: 1) Service logs: specify 'elasticsearch' or 'embeddings' to see logs from a specific service, 2) All logs: omit service parameter to see logs from all services, 3) Line limit: control how many recent log lines to retrieve (default: 100). Logs include: startup messages, health check results, API requests, error messages, performance metrics. Use this tool to: debug startup issues, investigate API errors, monitor performance, understand service behavior. Logs are retrieved from Docker container stdout/stderr."
    }

    fn emoji(&self) -> &'static str {
        "📋"
    }

    fn schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "service": {
                    "type": "string",
                    "description": "Optional: specific service to get logs from ('elasticsearch' or 'embeddings'). If not specified, returns logs from all services.",
                    "enum": ["elasticsearch", "embeddings"]
                },
                "lines": {
                    "type": "integer",
                    "description": "Optional: number of recent log lines to retrieve (default: 100, max: 1000)",
                    "minimum": 10,
                    "maximum": 1000,
                    "default": 100
                }
            },
            "additionalProperties": false,
            "description": "Get logs from RAG services for debugging"
        })
    }

    fn execute_impl(&self, id: u64, args: Option<Value>) {
        tokio::spawn(async move {
            let args: RagLogsArgs = args
                .and_then(|v| serde_json::from_value(v).ok())
                .unwrap_or_default();

            let stack = get_rag_stack();
            let stack_guard = stack.lock().await;

            match stack_guard.get_logs(args.service.as_deref()).await {
                Ok(logs) => {
                    let service_name = args.service.unwrap_or_else(|| "all services".to_string());
                    let lines_info = if let Some(lines) = args.lines {
                        format!(" (last {lines} lines)")
                    } else {
                        " (last 100 lines)".to_string()
                    };

                    let result = json!({
                        "content": [{
                            "type": "text",
                            "text": format!(
                                "📋 RAG Stack Logs - {}{}\n\n\
                                {}\n\n\
                                💡 Log Analysis Tips:\n\
                                - Look for ERROR or WARN messages\n\
                                - Check for 'healthy' status messages\n\
                                - Monitor API request patterns\n\
                                - Verify service startup sequences\n\
                                - Stack is auto-managed (starts/stops with editor)\n\
                                - Use rag_health for current status",
                                service_name,
                                lines_info,
                                if logs.trim().is_empty() {
                                    "📝 No recent logs available\n\n🔍 This might indicate:\n- Services haven't started yet\n- No recent activity\n- Logs have been rotated\n\n💡 Auto-start will occur on next RAG operation"
                                } else {
                                    &logs
                                }
                            )
                        }]
                    });
                    send_response(id, result);
                }
                Err(e) => send_error(id, -1, &format!("❌ Failed to get logs: {e}")),
            }
        });
    }
}

/// 📊 RAG Stack Status Tool
/// Provides detailed status information about the RAG stack
#[derive(Default)]
pub struct RagStatusTool;

impl Tool for RagStatusTool {
    fn name(&self) -> &'static str {
        "rag_status"
    }

    fn description(&self) -> &'static str {
        "Get comprehensive status information about the RAG stack including service details, resource usage, and configuration. This tool provides detailed insights into: 1) Service status: running/stopped state of each service, 2) Health metrics: detailed health check results and response times, 3) Configuration: current service configuration and parameters, 4) Resource usage: memory and CPU usage where available, 5) Network information: service URLs and port mappings, 6) Volume status: persistent data volume information. Use this tool to: understand current stack state, verify configuration, monitor resource usage, troubleshoot issues, get service URLs for direct access."
    }

    fn emoji(&self) -> &'static str {
        "📊"
    }

    fn schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {},
            "additionalProperties": false,
            "description": "Get detailed RAG stack status information - no parameters required"
        })
    }

    fn execute_impl(&self, id: u64, _args: Option<Value>) {
        tokio::spawn(async move {
            let status = get_rag_stack_status().await;
            let is_healthy = is_rag_stack_healthy().await;

            let result = json!({
                "content": [{
                    "type": "text",
                    "text": format!(
                        "📊 RAG Stack Status Report (Auto-Managed)\n\n\
                        🚀 Overall Status: {}\n\
                        🔍 Health Check: {}\n\
                        🤖 Lifecycle: Auto-managed (starts on first RAG operation)\n\n\
                        📋 Service Details:\n{}\n\n\
                        🌐 Service Endpoints:\n\
                        - 📊 Elasticsearch: http://localhost:9200\n\
                        - 🧠 Embeddings API: http://localhost:8000\n\
                        - 📚 API Documentation: http://localhost:8000/docs\n\
                        - 🔍 Health Checks: http://localhost:8000/health\n\n\
                        💾 Persistent Volumes:\n\
                        - es-data: Elasticsearch indices and data\n\
                        - model-cache: Embedding model storage\n\
                        - config-data: Service configuration\n\n\
                        🔧 Available Actions:\n\
                        - rag_health: Check current health\n\
                        - rag_logs: View service logs\n\
                        - rag_restart: Restart services\n\n\
                        💡 Auto-Lifecycle Notes:\n\
                        - Stack starts automatically on first RAG operation\n\
                        - Shuts down gracefully when editor exits\n\
                        - Data persists between editor sessions\n\
                        - No manual start/stop needed",
                        if is_healthy { "🟢 HEALTHY" } else { "🔴 UNHEALTHY" },
                        if is_healthy { "✅ All services healthy" } else { "❌ Some services unhealthy" },
                        serde_json::to_string_pretty(&status).unwrap_or_else(|_| "Status unavailable".to_string())
                    )
                }]
            });
            send_response(id, result);
        });
    }
}

/// 🛑 RAG Stack Stop Tool (Manual shutdown for development)
/// Manual shutdown for development/testing - normally auto-managed
#[derive(Default)]
pub struct RagStopTool;

impl Tool for RagStopTool {
    fn name(&self) -> &'static str {
        "rag_stop"
    }

    fn description(&self) -> &'static str {
        "Manually stop the RAG services stack for development/testing. Note: The RAG stack is normally auto-managed - it starts on first RAG operation and stops when the editor exits. This tool is primarily for development scenarios where you want to manually control the stack lifecycle. The stack will auto-start again on the next RAG operation."
    }

    fn emoji(&self) -> &'static str {
        "🛑"
    }

    fn schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {},
            "additionalProperties": false,
            "description": "Manually stop RAG stack (normally auto-managed)"
        })
    }

    fn execute_impl(&self, id: u64, _args: Option<Value>) {
        tokio::spawn(async move {
            match shutdown_rag_stack().await {
                Ok(_) => {
                    let result = json!({
                        "content": [{
                            "type": "text",
                            "text": "🛑 RAG Stack Manually Stopped\n\n\
                                    ✅ Elasticsearch gracefully shut down\n\
                                    ✅ Embeddings service stopped\n\
                                    ✅ Containers and networks removed\n\
                                    💾 Data volumes preserved\n\n\
                                    📋 Status: All services stopped\n\
                                    🤖 Auto-start: Will restart on next RAG operation\n\
                                    💡 Note: Stack is normally auto-managed\n\
                                    🔄 Data is safe and will be available on restart"
                        }]
                    });
                    send_response(id, result);
                }
                Err(e) => send_error(id, -1, &format!("❌ Failed to stop RAG stack: {e}")),
            }
        });
    }
}

// Register simplified RAG management tools (removed rag_start - now auto-managed)
register_tool!("rag_health", RagHealthTool);
register_tool!("rag_restart", RagRestartTool);
register_tool!("rag_logs", RagLogsTool);
register_tool!("rag_status", RagStatusTool);
register_tool!("rag_stop", RagStopTool); // Manual stop for development only
