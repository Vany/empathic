//! 🏎️ Performance monitoring and optimization for LSP communication
//!
//! Provides request queuing, response time metrics, and connection optimizations
//! to ensure optimal LSP server communication performance.

use std::collections::{HashMap, VecDeque};
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{oneshot, RwLock, Semaphore};
use serde_json::Value;

/// 📊 Performance metrics for LSP operations
#[derive(Debug)]
pub struct LspMetrics {
    /// Total requests processed
    pub total_requests: AtomicU64,
    /// Average response time in milliseconds
    pub avg_response_time_ms: AtomicU64,
    /// Current queue depth
    pub current_queue_depth: AtomicUsize,
    /// Peak queue depth
    pub peak_queue_depth: AtomicUsize,
    /// Successful requests
    pub successful_requests: AtomicU64,
    /// Failed requests
    pub failed_requests: AtomicU64,
    /// Cache hits
    pub cache_hits: AtomicU64,
    /// Cache misses
    pub cache_misses: AtomicU64,
}

impl Default for LspMetrics {
    fn default() -> Self {
        Self {
            total_requests: AtomicU64::new(0),
            avg_response_time_ms: AtomicU64::new(0),
            current_queue_depth: AtomicUsize::new(0),
            peak_queue_depth: AtomicUsize::new(0),
            successful_requests: AtomicU64::new(0),
            failed_requests: AtomicU64::new(0),
            cache_hits: AtomicU64::new(0),
            cache_misses: AtomicU64::new(0),
        }
    }
}

impl LspMetrics {
    /// 📈 Record a completed request
    pub fn record_request(&self, duration: Duration, success: bool) {
        let duration_ms = duration.as_millis() as u64;
        
        // Update counters
        self.total_requests.fetch_add(1, Ordering::Relaxed);
        if success {
            self.successful_requests.fetch_add(1, Ordering::Relaxed);
        } else {
            self.failed_requests.fetch_add(1, Ordering::Relaxed);
        }
        
        // Update rolling average response time
        let current_avg = self.avg_response_time_ms.load(Ordering::Relaxed);
        let total = self.total_requests.load(Ordering::Relaxed);
        let new_avg = if total == 1 {
            duration_ms
        } else {
            ((current_avg * (total - 1)) + duration_ms) / total
        };
        self.avg_response_time_ms.store(new_avg, Ordering::Relaxed);
    }
    
    /// 📊 Update queue depth metrics
    pub fn update_queue_depth(&self, depth: usize) {
        self.current_queue_depth.store(depth, Ordering::Relaxed);
        
        // Update peak if necessary
        let current_peak = self.peak_queue_depth.load(Ordering::Relaxed);
        if depth > current_peak {
            self.peak_queue_depth.store(depth, Ordering::Relaxed);
        }
    }
    
    /// 🎯 Record cache hit/miss
    pub fn record_cache_hit(&self, hit: bool) {
        if hit {
            self.cache_hits.fetch_add(1, Ordering::Relaxed);
        } else {
            self.cache_misses.fetch_add(1, Ordering::Relaxed);
        }
    }
    
    /// 📋 Get performance summary
    pub fn summary(&self) -> String {
        let total = self.total_requests.load(Ordering::Relaxed);
        let success = self.successful_requests.load(Ordering::Relaxed);
        let avg_ms = self.avg_response_time_ms.load(Ordering::Relaxed);
        let peak_queue = self.peak_queue_depth.load(Ordering::Relaxed);
        let hits = self.cache_hits.load(Ordering::Relaxed);
        let misses = self.cache_misses.load(Ordering::Relaxed);
        
        let success_rate = if total > 0 { (success * 100) / total } else { 0 };
        let cache_rate = if hits + misses > 0 { (hits * 100) / (hits + misses) } else { 0 };
        
        format!(
            "🏎️ LSP Performance: {} requests, {}% success, {}ms avg, peak queue: {}, {}% cache hit",
            total, success_rate, avg_ms, peak_queue, cache_rate
        )
    }
}

/// 🎯 Request priority levels for queue management
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum RequestPriority {
    /// 🔥 Critical - diagnostics, errors (highest priority)
    Critical = 0,
    /// ⚡ High - hover, completion (user-facing)
    High = 1,
    /// 📍 Medium - goto definition, references (navigation)
    Medium = 2,
    /// 📊 Low - symbols, workspace queries (background)
    Low = 3,
}

impl RequestPriority {
    /// Get priority for LSP method
    pub fn for_method(method: &str) -> Self {
        match method {
            "textDocument/publishDiagnostics" => Self::Critical,
            "textDocument/hover" | "textDocument/completion" => Self::High,
            "textDocument/definition" | "textDocument/references" => Self::Medium,
            "textDocument/documentSymbol" | "workspace/symbol" => Self::Low,
            _ => Self::Medium,
        }
    }
}

/// 📦 Queued request with metadata
#[derive(Debug)]
pub struct QueuedRequest {
    pub id: u64,
    pub method: String,
    pub params: Option<Value>,
    pub priority: RequestPriority,
    pub created_at: Instant,
    pub response_tx: oneshot::Sender<Result<Value, String>>,
}

/// 🚀 High-performance request queue with priority scheduling
pub struct RequestQueue {
    /// Priority queues (0 = highest priority)
    queues: [VecDeque<QueuedRequest>; 4],
    /// Request metrics
    metrics: Arc<LspMetrics>,
    /// Concurrent request limiter
    semaphore: Arc<Semaphore>,
}

impl RequestQueue {
    /// Create new request queue with concurrency limit
    pub fn new(max_concurrent: usize, metrics: Arc<LspMetrics>) -> Self {
        Self {
            queues: [
                VecDeque::new(), // Critical
                VecDeque::new(), // High  
                VecDeque::new(), // Medium
                VecDeque::new(), // Low
            ],
            metrics,
            semaphore: Arc::new(Semaphore::new(max_concurrent)),
        }
    }
    
    /// 📥 Enqueue request with automatic priority detection
    pub fn enqueue(&mut self, request: QueuedRequest) {
        let priority_idx = request.priority as usize;
        let method = request.method.clone(); // Clone for logging
        self.queues[priority_idx].push_back(request);
        
        // Update metrics
        let total_depth: usize = self.queues.iter().map(|q| q.len()).sum();
        self.metrics.update_queue_depth(total_depth);
        
        log::debug!("📥 Queued {} request, depth: {}", method, total_depth);
    }
    
    /// 📤 Dequeue highest priority request
    pub fn dequeue(&mut self) -> Option<QueuedRequest> {
        // Check queues in priority order
        for queue in &mut self.queues {
            if let Some(request) = queue.pop_front() {
                let total_depth: usize = self.queues.iter().map(|q| q.len()).sum();
                self.metrics.update_queue_depth(total_depth);
                
                log::debug!("📤 Dequeued {} request, remaining: {}", request.method, total_depth);
                return Some(request);
            }
        }
        None
    }
    
    /// 📊 Get queue statistics
    pub fn stats(&self) -> (usize, usize, usize, usize) {
        (
            self.queues[0].len(), // Critical
            self.queues[1].len(), // High
            self.queues[2].len(), // Medium  
            self.queues[3].len(), // Low
        )
    }
    
    /// 🎫 Acquire concurrency permit
    pub async fn acquire_permit(&self) -> Result<tokio::sync::SemaphorePermit<'_>, ()> {
        self.semaphore.acquire().await.map_err(|_| ())
    }
}

/// 🔧 Connection pool for LSP client reuse and optimization
#[derive(Debug)]
pub struct ConnectionPool {
    /// Active connections by project path
    connections: Arc<RwLock<HashMap<String, Arc<crate::lsp::client::LspClient>>>>,
    /// Performance metrics
    #[allow(dead_code)]
    metrics: Arc<LspMetrics>,
    /// Maximum connections per pool
    max_connections: usize,
}

impl ConnectionPool {
    /// Create new connection pool
    pub fn new(max_connections: usize, metrics: Arc<LspMetrics>) -> Self {
        Self {
            connections: Arc::new(RwLock::new(HashMap::new())),
            metrics,
            max_connections,
        }
    }
    
    /// 🔗 Get or create connection for project
    pub async fn get_connection(
        &self,
        project_path: &str,
    ) -> Option<Arc<crate::lsp::client::LspClient>> {
        let connections = self.connections.read().await;
        connections.get(project_path).cloned()
    }
    
    /// 💾 Store connection in pool
    pub async fn store_connection(
        &self,
        project_path: String,
        client: Arc<crate::lsp::client::LspClient>,
    ) -> Result<(), String> {
        let mut connections = self.connections.write().await;
        
        if connections.len() >= self.max_connections {
            // Remove oldest connection (simple LRU)
            if let Some(oldest_key) = connections.keys().next().cloned() {
                connections.remove(&oldest_key);
                log::info!("🗑️ Evicted connection for {} (pool full)", oldest_key);
            }
        }
        
        connections.insert(project_path.clone(), client);
        log::info!("💾 Stored connection for {} (pool size: {})", project_path, connections.len());
        Ok(())
    }
    
    /// 🧹 Remove connection from pool
    pub async fn remove_connection(&self, project_path: &str) {
        let mut connections = self.connections.write().await;
        if connections.remove(project_path).is_some() {
            log::info!("🗑️ Removed connection for {}", project_path);
        }
    }
    
    /// 📊 Get pool statistics
    pub async fn stats(&self) -> (usize, usize) {
        let connections = self.connections.read().await;
        (connections.len(), self.max_connections)
    }
}

/// 🏁 Performance test runner for LSP operations
#[derive(Debug)]
pub struct PerformanceTester {
    metrics: Arc<LspMetrics>,
}

impl PerformanceTester {
    pub fn new(metrics: Arc<LspMetrics>) -> Self {
        Self { metrics }
    }
    
    /// 🧪 Run performance benchmark
    pub async fn benchmark_request(
        &self,
        method: &str,
        operation: impl std::future::Future<Output = Result<Value, String>>,
    ) -> Result<Value, String> {
        let start = Instant::now();
        let result = operation.await;
        let duration = start.elapsed();
        
        // Record metrics
        self.metrics.record_request(duration, result.is_ok());
        
        // Log slow operations
        if duration > Duration::from_millis(500) {
            log::warn!("🐌 Slow LSP operation: {} took {}ms", method, duration.as_millis());
        } else {
            log::debug!("⚡ LSP operation: {} completed in {}ms", method, duration.as_millis());
        }
        
        result
    }
    
    /// 📊 Load test with concurrent requests
    pub async fn load_test(
        &self,
        concurrent_requests: usize,
        total_requests: usize,
        operation_factory: impl Fn() -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Value, String>> + Send>> + Send + Sync + 'static,
    ) -> String {
        let start = Instant::now();
        let operation_factory = Arc::new(operation_factory);
        let mut handles = Vec::new();
        
        let semaphore = Arc::new(Semaphore::new(concurrent_requests));
        
        for _i in 0..total_requests {
            let permit = semaphore.clone().acquire_owned().await.unwrap();
            let factory = operation_factory.clone();
            let metrics = self.metrics.clone();
            
            let handle = tokio::spawn(async move {
                let _permit = permit; // Hold permit for duration
                let start = Instant::now();
                let result = factory().await;
                let duration = start.elapsed();
                
                metrics.record_request(duration, result.is_ok());
                result
            });
            
            handles.push(handle);
        }
        
        // Wait for all requests to complete
        let mut success_count = 0;
        let mut _error_count = 0;
        
        for handle in handles {
            match handle.await {
                Ok(Ok(_)) => success_count += 1,
                Ok(Err(_)) => _error_count += 1,
                Err(_) => _error_count += 1,
            }
        }
        
        let total_duration = start.elapsed();
        let rps = total_requests as f64 / total_duration.as_secs_f64();
        
        format!(
            "🧪 Load test: {}/{} requests successful, {:.1} RPS, {}ms total",
            success_count,
            total_requests,
            rps,
            total_duration.as_millis()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_metrics_calculation() {
        let metrics = LspMetrics::default();
        
        // Record some requests
        metrics.record_request(Duration::from_millis(100), true);
        metrics.record_request(Duration::from_millis(200), true);
        metrics.record_request(Duration::from_millis(300), false);
        
        assert_eq!(metrics.total_requests.load(Ordering::Relaxed), 3);
        assert_eq!(metrics.successful_requests.load(Ordering::Relaxed), 2);
        assert_eq!(metrics.failed_requests.load(Ordering::Relaxed), 1);
        assert_eq!(metrics.avg_response_time_ms.load(Ordering::Relaxed), 200);
    }
    
    #[test]
    fn test_request_priority() {
        assert_eq!(RequestPriority::for_method("textDocument/publishDiagnostics"), RequestPriority::Critical);
        assert_eq!(RequestPriority::for_method("textDocument/hover"), RequestPriority::High);
        assert_eq!(RequestPriority::for_method("textDocument/definition"), RequestPriority::Medium);
        assert_eq!(RequestPriority::for_method("workspace/symbol"), RequestPriority::Low);
    }
    
    #[tokio::test]
    async fn test_request_queue() {
        let metrics = Arc::new(LspMetrics::default());
        let mut queue = RequestQueue::new(5, metrics);
        
        // Create test requests
        let (tx1, _rx1) = oneshot::channel();
        let (tx2, _rx2) = oneshot::channel();
        
        let req1 = QueuedRequest {
            id: 1,
            method: "textDocument/hover".to_string(),
            params: None,
            priority: RequestPriority::High,
            created_at: Instant::now(),
            response_tx: tx1,
        };
        
        let req2 = QueuedRequest {
            id: 2,
            method: "workspace/symbol".to_string(),
            params: None,
            priority: RequestPriority::Low,
            created_at: Instant::now(),
            response_tx: tx2,
        };
        
        // Enqueue low priority first, then high priority
        queue.enqueue(req2);
        queue.enqueue(req1);
        
        // High priority should come out first
        let dequeued = queue.dequeue().unwrap();
        assert_eq!(dequeued.id, 1);
        assert_eq!(dequeued.priority, RequestPriority::High);
        
        let dequeued = queue.dequeue().unwrap();
        assert_eq!(dequeued.id, 2);
        assert_eq!(dequeued.priority, RequestPriority::Low);
    }
}
