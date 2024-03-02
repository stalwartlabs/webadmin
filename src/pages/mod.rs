use chrono::{DateTime, Local, Utc};

pub mod directory;
pub mod login;
pub mod notfound;
pub mod queue;
pub mod reports;

pub fn maybe_plural(items: usize, singular: &str, plural: &str) -> String {
    if items == 1 {
        format!("{} {}", items, singular)
    } else {
        format!("{} {}", items, plural)
    }
}

pub trait FormatDateTime {
    fn format_date_time(&self) -> String;
    fn format_date(&self) -> String;
    fn format_time(&self) -> String;
}

impl FormatDateTime for DateTime<Utc> {
    fn format_date_time(&self) -> String {
        self.with_timezone(&Local)
            .format("%a, %d %b %Y %H:%M:%S")
            .to_string()
    }

    fn format_date(&self) -> String {
        self.with_timezone(&Local)
            .format("%a, %d %b %Y")
            .to_string()
    }

    fn format_time(&self) -> String {
        self.with_timezone(&Local).format("%H:%M:%S").to_string()
    }
}
