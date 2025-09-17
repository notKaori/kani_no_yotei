use anyhow::Result;
use chrono::{DateTime, Local, NaiveDateTime, TimeZone};

/// Parse various date/time formats
pub fn parse_due_date(date_str: &str) -> Result<DateTime<Local>> {
    let now = Local::now();
    
    // Try different formats
    let formats = [
        "%Y-%m-%d %H:%M",     // 2024-01-15 14:30
        "%Y-%m-%d",           // 2024-01-15
        "%m/%d/%Y %H:%M",     // 01/15/2024 14:30
        "%m/%d/%Y",           // 01/15/2024
        "%H:%M",              // 14:30 (today)
    ];
    
    for format in &formats {
        if let Ok(naive_dt) = NaiveDateTime::parse_from_str(date_str, format) {
            return Ok(Local.from_local_datetime(&naive_dt).single().unwrap_or(now));
        }
    }
    
    // Try parsing just date
    for format in &["%Y-%m-%d", "%m/%d/%Y"] {
        if let Ok(naive_date) = chrono::NaiveDate::parse_from_str(date_str, format) {
            let naive_dt = naive_date.and_hms_opt(9, 0, 0).unwrap(); // Default to 9 AM
            return Ok(Local.from_local_datetime(&naive_dt).single().unwrap_or(now));
        }
    }
    
    // Try parsing just time (assume today)
    if let Ok(naive_time) = chrono::NaiveTime::parse_from_str(date_str, "%H:%M") {
        let naive_dt = now.date_naive().and_time(naive_time);
        return Ok(Local.from_local_datetime(&naive_dt).single().unwrap_or(now));
    }
    
    // Handle relative dates
    match date_str.to_lowercase().as_str() {
        "tomorrow" => Ok(now + chrono::Duration::days(1)),
        "today" => Ok(now),
        _ => Err(anyhow::anyhow!("Unable to parse date: {}", date_str)),
    }
}
