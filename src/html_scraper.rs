use std::{collections::HashMap, fmt::{self, Debug, Formatter}, sync::Arc};

use scraper::Html;

use crate::{cleaner::TextCleaner, scraper_config::ScrapeConfig, visitor::{ScraperVisitor, Visitor}, ConfigError};


/// A builder for the `HtmlScraper` struct
/// That allows for configuring the scraper
/// before building it
pub struct HtmlScraperBuilder {
    config: Option<String>,
    cleaner: Option<Arc<dyn TextCleaner>>,
}

impl HtmlScraperBuilder {
    pub fn new() -> Self {
        HtmlScraperBuilder {
            config: None,
            cleaner: None,
        }
    }

    pub fn with_config(mut self, config: &str) -> Self {
        self.config = Some(config.to_string());
        self
    }

    pub fn with_cleaner<T: TextCleaner + 'static>(mut self, cleaner: T) -> Self {
        self.cleaner = Some(Arc::new(cleaner));
        self
    }

    pub fn build(self) -> HtmlScraper {
        HtmlScraper {
            config: self.config,
            cleaner: self.cleaner,
        }
    }
}


/// A struct that can scrape HTML documents
/// 
/// # Example
/// 
/// ```
/// use scraper::{Html, Selector};
/// use serde_json::{Map, Value};
/// use serde::{Deserialize, Serialize};
/// 
/// // Scraping configuration
/// 
/// 
/// ```
#[derive(Clone)]
pub struct HtmlScraper {
    config: Option<String>,
    cleaner: Option<Arc<dyn TextCleaner>>,
}

impl Debug for HtmlScraper {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "HtmlScraper")
    }
}

impl HtmlScraper {
    pub fn new() -> HtmlScraperBuilder {
        HtmlScraperBuilder::new()
    }
    pub fn scrape<T: ScrapeConfig + for<'a> From<HashMap<String, String>>>(
        &self,
        html: &str,
    ) -> Result<T, ConfigError> {
        let scraper_config = if let Some(config_str) = &self.config {
            T::from_config(&config_str)?
        } else {
            T::get_config()
        };

        let document = Html::parse_document(html);
        let mut visitor = ScraperVisitor;
        let mut result = HashMap::new();

        for rule in scraper_config.rules {
            result.extend(visitor.visit_element(
                &document.root_element(),
                &rule,
                self.cleaner.as_deref(),
            ));
        }

        Ok(T::from(result))
    }
}

impl Default for HtmlScraper {
    fn default() -> Self {
        HtmlScraper {
            config: None,
            cleaner: None,
        }
    }
}