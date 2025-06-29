use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Summary information for a performance report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportSummary {
    pub id: String,
    pub name: String,
    pub test_type: String,
    pub timestamp: DateTime<Utc>,
    pub duration: u64, // in milliseconds
    pub coverage_percentage: f64,
    pub status: String, // "completed", "failed", "partial"
}

/// Detailed performance report data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportDetail {
    pub id: String,
    pub name: String,
    pub test_type: String,
    pub timestamp: DateTime<Utc>,
    pub duration: u64,
    pub status: String,
    pub performance_data: serde_json::Value,
    pub coverage_data: Option<CoverageData>,
    pub summary: String,
    pub files: Vec<String>,
}

/// Coverage data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoverageData {
    #[serde(rename = "testInfo")]
    pub test_info: TestInfo,
    #[serde(rename = "endpointCoverage")]
    pub endpoint_coverage: HashMap<String, EndpointStats>,
    pub summary: CoverageSummary,
}

/// Test information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestInfo {
    pub timestamp: String,
    pub duration: f64,
    pub iterations: u64,
    pub vus: u64,
}

/// Endpoint statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EndpointStats {
    pub hits: u64,
    #[serde(rename = "successRate")]
    pub success_rate: String,
    #[serde(rename = "avgResponseTime")]
    pub avg_response_time: String,
    pub errors: u64,
    pub tested: bool,
}

/// Coverage summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoverageSummary {
    #[serde(rename = "totalEndpoints")]
    pub total_endpoints: u64,
    #[serde(rename = "testedEndpoints")]
    pub tested_endpoints: u64,
    #[serde(rename = "coveragePercentage")]
    pub coverage_percentage: String,
}

/// Performance metrics extracted from k6 data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub total_requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub success_rate: f64,
    pub avg_response_time: f64,
    pub p95_response_time: f64,
    pub p99_response_time: f64,
    pub requests_per_second: f64,
    pub test_duration: f64,
}

impl ReportDetail {
    /// Extract performance metrics from the raw data
    pub fn get_performance_metrics(&self) -> Option<PerformanceMetrics> {
        let metrics = self.performance_data.get("metrics")?;

        // Extract key metrics from k6 data structure
        let http_reqs = metrics.get("http_reqs")?;
        let http_req_duration = metrics.get("http_req_duration")?;
        let http_req_failed = metrics.get("http_req_failed")?;

        let total_requests = http_reqs.get("count")?.as_u64()?;
        let failed_rate = http_req_failed.get("rate")?.as_f64()?;
        let successful_requests = ((1.0 - failed_rate) * total_requests as f64) as u64;
        let failed_requests = total_requests - successful_requests;

        let avg_response_time = http_req_duration.get("avg")?.as_f64()?;
        let p95_response_time = http_req_duration.get("p(95)")?.as_f64()?;
        let p99_response_time = http_req_duration.get("p(99)")?.as_f64()?;

        let requests_per_second = http_reqs.get("rate")?.as_f64()?;
        let test_duration = self.duration as f64 / 1000.0; // Convert ms to seconds

        Some(PerformanceMetrics {
            total_requests,
            successful_requests,
            failed_requests,
            success_rate: (1.0 - failed_rate) * 100.0,
            avg_response_time,
            p95_response_time,
            p99_response_time,
            requests_per_second,
            test_duration,
        })
    }

    /// Get status color for UI
    pub fn status_color(&self) -> &'static str {
        match self.status.as_str() {
            "completed" => "success",
            "failed" => "danger",
            "partial" => "warning",
            _ => "secondary",
        }
    }

    /// Get formatted duration
    pub fn formatted_duration(&self) -> String {
        let seconds = self.duration / 1000;
        if seconds < 60 {
            format!("{}s", seconds)
        } else {
            format!("{}m {}s", seconds / 60, seconds % 60)
        }
    }

    /// Get formatted timestamp
    pub fn formatted_timestamp(&self) -> String {
        self.timestamp.format("%Y-%m-%d %H:%M:%S UTC").to_string()
    }
}

impl ReportSummary {
    /// Get status color for UI
    pub fn status_color(&self) -> &'static str {
        match self.status.as_str() {
            "completed" => "success",
            "failed" => "danger",
            "partial" => "warning",
            _ => "secondary",
        }
    }

    /// Get formatted duration
    pub fn formatted_duration(&self) -> String {
        let seconds = self.duration / 1000;
        if seconds < 60 {
            format!("{}s", seconds)
        } else {
            format!("{}m {}s", seconds / 60, seconds % 60)
        }
    }

    /// Get formatted timestamp
    pub fn formatted_timestamp(&self) -> String {
        self.timestamp.format("%Y-%m-%d %H:%M:%S").to_string()
    }

    /// Get coverage color based on percentage
    pub fn coverage_color(&self) -> &'static str {
        if self.coverage_percentage >= 90.0 {
            "success"
        } else if self.coverage_percentage >= 70.0 {
            "warning"
        } else {
            "danger"
        }
    }
}

impl EndpointStats {
    /// Get success rate as float
    pub fn success_rate_float(&self) -> f64 {
        self.success_rate
            .trim_end_matches('%')
            .parse()
            .unwrap_or(0.0)
    }

    /// Get average response time as float (in ms)
    pub fn avg_response_time_float(&self) -> f64 {
        self.avg_response_time
            .trim_end_matches("ms")
            .parse()
            .unwrap_or(0.0)
    }

    /// Get status color based on success rate
    pub fn status_color(&self) -> &'static str {
        let rate = self.success_rate_float();
        if rate >= 95.0 {
            "success"
        } else if rate >= 80.0 {
            "warning"
        } else {
            "danger"
        }
    }

    /// Get response time color based on performance
    pub fn response_time_color(&self) -> &'static str {
        let time = self.avg_response_time_float();
        if time <= 100.0 {
            "success"
        } else if time <= 500.0 {
            "warning"
        } else {
            "danger"
        }
    }
}
