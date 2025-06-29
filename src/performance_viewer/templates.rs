use crate::performance_viewer::models::*;
use askama::Template;

/// Base template with common layout
#[derive(Template)]
#[template(path = "base.html")]
pub struct BaseTemplate<'a> {
    pub title: &'a str,
    pub content: &'a str,
}

/// Index/home page template
#[derive(Template)]
#[template(path = "index.html")]
pub struct IndexTemplate {
    pub title: String,
    pub description: String,
}

/// Report list template
#[derive(Template)]
#[template(path = "report_list.html")]
pub struct ReportListTemplate {
    pub title: String,
    pub reports: Vec<ReportSummary>,
    pub current_sort: String,
    pub current_filter: String,
}

/// Report detail template
#[derive(Template)]
#[template(path = "report_detail.html")]
pub struct ReportDetailTemplate {
    pub title: String,
    pub report: ReportDetail,
}

/// Coverage details template
#[derive(Template)]
#[template(path = "coverage.html")]
pub struct CoverageTemplate {
    pub title: String,
    pub report_id: String,
    pub coverage: CoverageData,
}

/// Raw data template
#[derive(Template)]
#[template(path = "raw_data.html")]
pub struct RawDataTemplate {
    pub title: String,
    pub report_id: String,
    pub raw_data: serde_json::Value,
}
