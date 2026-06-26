#[derive(Debug, Clone)]
pub struct Telemetry {
    pub llm_calls: u64,
    pub tokens_used: u64,
    pub algorithm_hits: u64,
    pub cache_hits: u64,
    pub local_search_successes: u64,
    avg_latency_ms: f64,
}

impl Telemetry {
    pub fn new() -> Self {
        Telemetry {
            llm_calls: 0,
            tokens_used: 0,
            algorithm_hits: 0,
            cache_hits: 0,
            local_search_successes: 0,
            avg_latency_ms: 0.0,
        }
    }

    pub fn record_llm_call(&mut self, tokens: u64, latency_ms: u64) {
        self.llm_calls += 1;
        self.tokens_used += tokens;
        let n = self.llm_calls as f64;
        self.avg_latency_ms = (self.avg_latency_ms * (n - 1.0) + latency_ms as f64) / n;
    }

    pub fn record_algorithm_hit(&mut self) {
        self.algorithm_hits += 1;
    }
    pub fn record_cache_hit(&mut self) {
        self.cache_hits += 1;
    }
    pub fn record_local_search_success(&mut self) {
        self.local_search_successes += 1;
    }

    pub fn llm_avoidance_rate(&self) -> f32 {
        let total = self.llm_calls + self.algorithm_hits + self.cache_hits + self.local_search_successes;
        if total == 0 {
            return 0.0;
        }
        (self.algorithm_hits + self.cache_hits + self.local_search_successes) as f32 / total as f32
    }

    pub fn summary(&self) -> String {
        format!(
            "LLM: {} calls | avoided: {} | avoidance: {:.1}% | tokens: {:.1}M",
            self.llm_calls,
            self.algorithm_hits + self.cache_hits + self.local_search_successes,
            self.llm_avoidance_rate() * 100.0,
            self.tokens_used as f64 / 1_000_000.0,
        )
    }
}
