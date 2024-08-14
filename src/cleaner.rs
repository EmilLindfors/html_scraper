// New trait for text cleaning
pub trait TextCleaner: Send + Sync {
    fn clean(&self, text: &str) -> String;
}

// Default text cleaner that removes newlines and extra whitespace
pub struct DefaultCleaner;

impl TextCleaner for DefaultCleaner {
    fn clean(&self, text: &str) -> String {
        text.lines()
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .collect::<Vec<&str>>()
            .join(" ")
    }
}
