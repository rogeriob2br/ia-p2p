//! Output formatting for MVP CLI
//!
//! Supports multiple output formats:
//! - Table (pretty-printed with colors)
//! - JSON (machine-readable)
//! - CSV (for spreadsheet import)
//! - Plain (minimal formatting)

use clap::ValueEnum;
use colored::Colorize;
use comfy_table::{Table, Row, Cell, ContentArrangement, presets};
use serde::Serialize;

/// Output format options
#[derive(Debug, Clone, ValueEnum, Default)]
pub enum OutputFormat {
    #[default]
    Table,
    Json,
    Csv,
    Plain,
}

/// Output formatter
pub struct OutputFormatter {
    format: OutputFormat,
    colors_enabled: bool,
}

impl OutputFormatter {
    /// Create new formatter
    pub fn new(format: OutputFormat, colors_enabled: bool) -> Self {
        Self { format, colors_enabled }
    }
    
    /// Format data as table
    pub fn table(&self, headers: Vec<&str>, rows: Vec<Vec<String>>) -> String {
        match self.format {
            OutputFormat::Table => self.format_table(headers, rows),
            OutputFormat::Json => self.format_table_as_json(headers, rows),
            OutputFormat::Csv => self.format_csv(headers, rows),
            OutputFormat::Plain => self.format_plain(headers, rows),
        }
    }
    
    /// Format any serializable data
    pub fn data<T: Serialize>(&self, data: &T) -> String {
        match self.format {
            OutputFormat::Json => {
                serde_json::to_string_pretty(data).unwrap_or_else(|_| "{}".to_string())
            }
            _ => {
                serde_json::to_string_pretty(data).unwrap_or_else(|_| "{}".to_string())
            }
        }
    }
    
    /// Format success message
    pub fn success(&self, message: &str) -> String {
        if self.colors_enabled {
            format!("{} {}", "✓".green(), message)
        } else {
            format!("[OK] {}", message)
        }
    }
    
    /// Format error message
    pub fn error(&self, message: &str) -> String {
        if self.colors_enabled {
            format!("{} {}", "✗".red(), message)
        } else {
            format!("[ERROR] {}", message)
        }
    }
    
    /// Format warning message
    pub fn warning(&self, message: &str) -> String {
        if self.colors_enabled {
            format!("{} {}", "⚠".yellow(), message)
        } else {
            format!("[WARN] {}", message)
        }
    }
    
    /// Format info message
    pub fn info(&self, message: &str) -> String {
        if self.colors_enabled {
            format!("{} {}", "ℹ".blue(), message)
        } else {
            format!("[INFO] {}", message)
        }
    }
    
    /// Format a key-value pair
    pub fn kv(&self, key: &str, value: &str) -> String {
        if self.colors_enabled {
            format!("{}: {}", key.cyan(), value)
        } else {
            format!("{}: {}", key, value)
        }
    }
    
    /// Format a header/title
    pub fn header(&self, text: &str) -> String {
        if self.colors_enabled {
            format!("\n{}\n{}", text.bold(), "─".repeat(text.len()))
        } else {
            format!("\n{}\n{}", text, "-".repeat(text.len()))
        }
    }
    
    /// Format status indicator
    pub fn status(&self, status: &str) -> String {
        if self.colors_enabled {
            match status.to_lowercase().as_str() {
                "online" | "running" | "completed" | "success" | "healthy" => {
                    status.green().to_string()
                }
                "offline" | "stopped" | "failed" | "error" | "unhealthy" => {
                    status.red().to_string()
                }
                "pending" | "processing" | "starting" => {
                    status.yellow().to_string()
                }
                _ => status.to_string()
            }
        } else {
            status.to_string()
        }
    }
    
    // Private formatting methods
    
    fn format_table(&self, headers: Vec<&str>, rows: Vec<Vec<String>>) -> String {
        let mut table = Table::new();
        table.load_preset(presets::UTF8_FULL);
        table.set_content_arrangement(ContentArrangement::Dynamic);
        
        // Add headers
        let header_row: Vec<Cell> = headers.iter()
            .map(|h| {
                if self.colors_enabled {
                    Cell::new(h.bold().to_string())
                } else {
                    Cell::new(*h)
                }
            })
            .collect();
        table.set_header(header_row);
        
        // Add rows
        for row_data in rows {
            let row: Vec<Cell> = row_data.iter()
                .map(|c| Cell::new(c))
                .collect();
            table.add_row(row);
        }
        
        table.to_string()
    }
    
    fn format_table_as_json(&self, headers: Vec<&str>, rows: Vec<Vec<String>>) -> String {
        let data: Vec<serde_json::Value> = rows.iter()
            .map(|row| {
                let mut obj = serde_json::Map::new();
                for (i, cell) in row.iter().enumerate() {
                    if let Some(header) = headers.get(i) {
                        obj.insert(header.to_string(), serde_json::Value::String(cell.clone()));
                    }
                }
                serde_json::Value::Object(obj)
            })
            .collect();
        
        serde_json::to_string_pretty(&data).unwrap_or_else(|_| "[]".to_string())
    }
    
    fn format_csv(&self, headers: Vec<&str>, rows: Vec<Vec<String>>) -> String {
        let mut output = headers.join(",");
        output.push('\n');
        
        for row in rows {
            let escaped: Vec<String> = row.iter()
                .map(|cell| {
                    if cell.contains(',') || cell.contains('"') || cell.contains('\n') {
                        format!("\"{}\"", cell.replace('"', "\"\""))
                    } else {
                        cell.clone()
                    }
                })
                .collect();
            output.push_str(&escaped.join(","));
            output.push('\n');
        }
        
        output
    }
    
    fn format_plain(&self, headers: Vec<&str>, rows: Vec<Vec<String>>) -> String {
        let mut output = String::new();
        
        for row in rows {
            for (i, cell) in row.iter().enumerate() {
                if let Some(header) = headers.get(i) {
                    output.push_str(&format!("{}: {}\n", header, cell));
                }
            }
            output.push('\n');
        }
        
        output
    }
}

/// Progress indicator
pub struct ProgressIndicator {
    spinner: indicatif::ProgressBar,
}

impl ProgressIndicator {
    /// Create a new spinner
    pub fn spinner(message: &str) -> Self {
        let spinner = indicatif::ProgressBar::new_spinner();
        spinner.set_style(
            indicatif::ProgressStyle::default_spinner()
                .template("{spinner:.green} {msg}")
                .unwrap()
        );
        spinner.set_message(message.to_string());
        spinner.enable_steady_tick(std::time::Duration::from_millis(100));
        
        Self { spinner }
    }
    
    /// Create a progress bar
    pub fn bar(total: u64, message: &str) -> Self {
        let bar = indicatif::ProgressBar::new(total);
        bar.set_style(
            indicatif::ProgressStyle::default_bar()
                .template("{spinner:.green} [{bar:40.cyan/blue}] {pos}/{len} {msg}")
                .unwrap()
                .progress_chars("=>-")
        );
        bar.set_message(message.to_string());
        
        Self { spinner: bar }
    }
    
    /// Update message
    pub fn set_message(&self, message: &str) {
        self.spinner.set_message(message.to_string());
    }
    
    /// Increment progress
    pub fn inc(&self, delta: u64) {
        self.spinner.inc(delta);
    }
    
    /// Finish with message
    pub fn finish(&self, message: &str) {
        self.spinner.finish_with_message(message.to_string());
    }
    
    /// Finish and clear
    pub fn finish_and_clear(&self) {
        self.spinner.finish_and_clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_table_format() {
        let formatter = OutputFormatter::new(OutputFormat::Table, false);
        let output = formatter.table(
            vec!["Name", "Value"],
            vec![vec!["test".to_string(), "123".to_string()]]
        );
        assert!(output.contains("Name"));
        assert!(output.contains("test"));
    }
    
    #[test]
    fn test_json_format() {
        let formatter = OutputFormatter::new(OutputFormat::Json, false);
        let output = formatter.table(
            vec!["name", "value"],
            vec![vec!["test".to_string(), "123".to_string()]]
        );
        assert!(output.contains("\"name\""));
        assert!(output.contains("\"test\""));
    }
    
    #[test]
    fn test_csv_format() {
        let formatter = OutputFormatter::new(OutputFormat::Csv, false);
        let output = formatter.table(
            vec!["name", "value"],
            vec![vec!["test".to_string(), "123".to_string()]]
        );
        assert!(output.contains("name,value"));
        assert!(output.contains("test,123"));
    }
}











