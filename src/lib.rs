mod cleaner;
mod scraper_config;
mod visitor;
mod html_scraper;
mod error;

pub use cleaner::{DefaultCleaner, TextCleaner};
pub use scraper_config::{ScrapeRule, ScraperConfig};
pub use visitor::{ScraperVisitor, Visitor};
pub use html_scraper::{HtmlScraper, HtmlScraperBuilder};
pub use error::ConfigError;