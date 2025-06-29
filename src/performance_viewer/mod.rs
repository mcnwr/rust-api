use askama::Template;
use axum::{
    extract::{Path, Query},
    http::StatusCode,
    response::{Html, IntoResponse},
    routing::get,
    Router,
};
use chrono::{DateTime, TimeZone, Utc};
use serde::Deserialize;
use std::fs;
use std::path::PathBuf;
use tower_http::services::ServeDir;

pub mod models;
pub mod templates;

use models::*;
use templates::*;

#[derive(Debug, Deserialize)]
pub struct ListParams {
    sort: Option<String>,
    filter: Option<String>,
}

/// Create performance viewer router
pub fn create_router() -> Router {
    Router::new()
        .route("/", get(index))
        .route("/reports", get(list_reports))
        .route("/reports/:report_id", get(view_report))
        .route("/reports/:report_id/coverage", get(view_coverage))
        .route("/reports/:report_id/raw", get(view_raw_data))
        .route("/api/reports", get(api_list_reports))
        .route("/api/reports/:report_id", get(api_get_report))
        .nest_service("/static", ServeDir::new("static"))
}

/// Index page handler
async fn index() -> impl IntoResponse {
    let template = IndexTemplate {
        title: "K6 Performance Reports".to_string(),
        description: "Monitor and analyze your k6 performance test results with beautiful visualizations and detailed coverage analysis.".to_string(),
    };

    Html(
        template
            .render()
            .unwrap_or_else(|e| format!("Template error: {}", e)),
    )
}

/// List all reports
async fn list_reports(Query(params): Query<ListParams>) -> impl IntoResponse {
    let reports = load_reports().await;
    let mut filtered_reports = reports;

    // Apply filter
    if let Some(filter) = &params.filter {
        if !filter.is_empty() {
            filtered_reports.retain(|r| {
                r.name.to_lowercase().contains(&filter.to_lowercase())
                    || r.test_type.to_lowercase().contains(&filter.to_lowercase())
            });
        }
    }

    // Apply sorting
    let sort = params.sort.unwrap_or_else(|| "date".to_string());
    match sort.as_str() {
        "name" => filtered_reports.sort_by(|a, b| a.name.cmp(&b.name)),
        "duration" => filtered_reports.sort_by(|a, b| a.duration.cmp(&b.duration)),
        _ => filtered_reports.sort_by(|a, b| b.timestamp.cmp(&a.timestamp)), // date, newest first
    }

    let template = ReportListTemplate {
        title: "Performance Reports".to_string(),
        reports: filtered_reports,
        current_sort: sort,
        current_filter: params.filter.unwrap_or_default(),
    };

    Html(
        template
            .render()
            .unwrap_or_else(|e| format!("Template error: {}", e)),
    )
}

/// View detailed report
async fn view_report(Path(report_id): Path<String>) -> impl IntoResponse {
    match load_report_detail(&report_id).await {
        Some(report) => {
            let template = ReportDetailTemplate {
                title: format!("Report: {}", report.name),
                report,
            };
            Html(
                template
                    .render()
                    .unwrap_or_else(|e| format!("Template error: {}", e)),
            )
        }
        None => Html("<h1>Report not found</h1>".to_string()),
    }
}

/// View coverage details
async fn view_coverage(Path(report_id): Path<String>) -> impl IntoResponse {
    match load_report_detail(&report_id).await {
        Some(report) => {
            if let Some(coverage) = report.coverage_data {
                let template = CoverageTemplate {
                    title: format!("Coverage: {}", report.name),
                    report_id,
                    coverage,
                };
                Html(
                    template
                        .render()
                        .unwrap_or_else(|e| format!("Template error: {}", e)),
                )
            } else {
                Html("<h1>No coverage data available</h1>".to_string())
            }
        }
        None => Html("<h1>Report not found</h1>".to_string()),
    }
}

/// View raw data
async fn view_raw_data(Path(report_id): Path<String>) -> impl IntoResponse {
    match load_report_detail(&report_id).await {
        Some(report) => {
            let template = RawDataTemplate {
                title: format!("Raw Data: {}", report.name),
                report_id,
                raw_data: report.performance_data,
            };
            Html(
                template
                    .render()
                    .unwrap_or_else(|e| format!("Template error: {}", e)),
            )
        }
        None => Html("<h1>Report not found</h1>".to_string()),
    }
}

/// API endpoint to list reports
async fn api_list_reports() -> impl IntoResponse {
    let reports = load_reports().await;
    axum::Json(reports)
}

/// API endpoint to get specific report
async fn api_get_report(Path(report_id): Path<String>) -> impl IntoResponse {
    match load_report_detail(&report_id).await {
        Some(report) => (StatusCode::OK, axum::Json(report)).into_response(),
        None => (
            StatusCode::NOT_FOUND,
            axum::Json(serde_json::json!({"error": "Report not found"})),
        )
            .into_response(),
    }
}

/// Load all report summaries from the reports directory
async fn load_reports() -> Vec<ReportSummary> {
    let mut reports = Vec::new();
    let reports_dir = PathBuf::from("reports");

    if !reports_dir.exists() {
        return reports;
    }

    if let Ok(entries) = fs::read_dir(&reports_dir) {
        for entry in entries.flatten() {
            if let Ok(file_type) = entry.file_type() {
                if file_type.is_dir() {
                    if let Some(report_id) = entry.file_name().to_str() {
                        if let Some(summary) = load_report_summary(report_id).await {
                            reports.push(summary);
                        }
                    }
                }
            }
        }
    }

    // Sort by timestamp, newest first
    reports.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
    reports
}

/// Load report summary from directory
async fn load_report_summary(report_id: &str) -> Option<ReportSummary> {
    let summary_file = PathBuf::from("reports")
        .join(report_id)
        .join("performance-summary.txt");

    let coverage_file = PathBuf::from("reports")
        .join(report_id)
        .join("endpoint-coverage.json");

    // Try to extract information from summary file
    if let Ok(summary_content) = fs::read_to_string(&summary_file) {
        // Parse timestamp from report directory name or file metadata
        let timestamp = extract_timestamp_from_id(report_id).unwrap_or_else(|| Utc::now());

        // Extract test duration and other metrics from summary
        let duration = extract_duration_from_summary(&summary_content);
        let status = if summary_content.contains("✅") || summary_content.contains("Test completed")
        {
            "completed".to_string()
        } else if summary_content.contains("❌") || summary_content.contains("failed") {
            "failed".to_string()
        } else {
            "partial".to_string()
        };

        // Load coverage percentage
        let coverage_percentage = if let Ok(coverage_content) = fs::read_to_string(&coverage_file) {
            extract_coverage_percentage(&coverage_content).unwrap_or(0.0)
        } else {
            0.0
        };

        Some(ReportSummary {
            id: report_id.to_string(),
            name: format!("Performance Test {}", report_id),
            test_type: "comprehensive".to_string(),
            timestamp,
            duration,
            coverage_percentage,
            status,
        })
    } else {
        None
    }
}

/// Load detailed report data
async fn load_report_detail(report_id: &str) -> Option<ReportDetail> {
    let report_dir = PathBuf::from("reports").join(report_id);

    if !report_dir.exists() {
        return None;
    }

    // Load summary first
    let summary_info = load_report_summary(report_id).await?;

    // Load performance data
    let performance_data_file = report_dir.join("performance-data.json");
    let performance_data = if let Ok(content) = fs::read_to_string(&performance_data_file) {
        serde_json::from_str(&content).unwrap_or(serde_json::json!({}))
    } else {
        serde_json::json!({})
    };

    // Load coverage data
    let coverage_data_file = report_dir.join("endpoint-coverage.json");
    let coverage_data = if let Ok(content) = fs::read_to_string(&coverage_data_file) {
        serde_json::from_str::<CoverageData>(&content).ok()
    } else {
        None
    };

    // Load summary text
    let summary_file = report_dir.join("performance-summary.txt");
    let summary =
        fs::read_to_string(&summary_file).unwrap_or_else(|_| "No summary available".to_string());

    // List generated files
    let mut files = Vec::new();
    if let Ok(entries) = fs::read_dir(&report_dir) {
        for entry in entries.flatten() {
            if let Some(name) = entry.file_name().to_str() {
                files.push(name.to_string());
            }
        }
    }
    files.sort();

    Some(ReportDetail {
        id: summary_info.id,
        name: summary_info.name,
        test_type: summary_info.test_type,
        timestamp: summary_info.timestamp,
        duration: summary_info.duration,
        status: summary_info.status,
        performance_data,
        coverage_data,
        summary,
        files,
    })
}

/// Extract timestamp from report ID (format: YYYYMMDD_HHMMSS)
fn extract_timestamp_from_id(report_id: &str) -> Option<DateTime<Utc>> {
    if let Some(timestamp_part) = report_id.split('_').next() {
        if timestamp_part.len() >= 8 {
            let year: i32 = timestamp_part[0..4].parse().ok()?;
            let month: u32 = timestamp_part[4..6].parse().ok()?;
            let day: u32 = timestamp_part[6..8].parse().ok()?;

            let (hour, minute, second) = if report_id.len() > 9 {
                let time_part = &report_id[9..];
                if time_part.len() >= 6 {
                    (
                        time_part[0..2].parse().unwrap_or(0),
                        time_part[2..4].parse().unwrap_or(0),
                        time_part[4..6].parse().unwrap_or(0),
                    )
                } else {
                    (0, 0, 0)
                }
            } else {
                (0, 0, 0)
            };

            Utc.with_ymd_and_hms(year, month, day, hour, minute, second)
                .single()
        } else {
            None
        }
    } else {
        None
    }
}

/// Extract test duration from summary content
fn extract_duration_from_summary(content: &str) -> u64 {
    // Look for duration patterns in the summary
    for line in content.lines() {
        if line.contains("Duration:") || line.contains("Test duration:") {
            // Extract numbers from the line
            let numbers: Vec<u64> = line
                .split_whitespace()
                .filter_map(|s| {
                    s.chars()
                        .filter(|c| c.is_numeric())
                        .collect::<String>()
                        .parse()
                        .ok()
                })
                .collect();

            if let Some(&duration) = numbers.first() {
                return duration * 1000; // Convert to milliseconds
            }
        }
    }

    // Default fallback
    60000 // 60 seconds
}

/// Extract coverage percentage from coverage JSON
fn extract_coverage_percentage(content: &str) -> Option<f64> {
    if let Ok(data) = serde_json::from_str::<serde_json::Value>(content) {
        data.get("summary")
            .and_then(|s| s.get("coverage_percentage"))
            .and_then(|p| p.as_str())
            .and_then(|s| s.trim_end_matches('%').parse().ok())
    } else {
        None
    }
}
