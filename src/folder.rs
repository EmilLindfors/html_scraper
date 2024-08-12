use scraper::{Html, Selector};
use serde_json::{Map, Value};
use serde::{Deserialize, Serialize};

// Scraping configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ScrapeRule {
    One {
        selector: String,
        name: Option<String>,
        sub_rules: Option<Vec<ScrapeRule>>,
        attribute: Option<String>,
    },
    All {
        selector: String,
        name: Option<String>,
        sub_rules: Option<Vec<ScrapeRule>>,
        attribute: Option<String>,
    },
    Text {
        selector: String,
        name: Option<String>,
    },
}

// Trait for types that can be scraped
pub trait Scrapable {
    fn scrape_rules() -> Vec<ScrapeRule>;
}

// Example implementation for a news article
pub struct NewsArticle;

impl Scrapable for NewsArticle {
    fn scrape_rules() -> Vec<ScrapeRule> {
        vec![
            ScrapeRule::One {
                selector: "h1.title".to_string(),
                name: Some("title".to_string()),
                sub_rules: None,
                attribute: None,
            },
            ScrapeRule::One {
                selector: "div.author".to_string(),
                name: Some("author".to_string()),
                sub_rules: None,
                attribute: None,
            },
            ScrapeRule::All {
                selector: "div.paragraph".to_string(),
                name: Some("content".to_string()),
                sub_rules: None,
                attribute: None,
            },
        ]
    }
}

// The abstract folder
pub trait Folder {
    fn fold_rule(&mut self, rule: &ScrapeRule, element: &scraper::ElementRef) -> Value;
    
    fn fold_one(&mut self, selector: &str, name: &Option<String>, sub_rules: &Option<Vec<ScrapeRule>>, attribute: &Option<String>, element: &scraper::ElementRef) -> Value;
    
    fn fold_all(&mut self, selector: &str, name: &Option<String>, sub_rules: &Option<Vec<ScrapeRule>>, attribute: &Option<String>, element: &scraper::ElementRef) -> Value;
    
    fn fold_text(&mut self, selector: &str, name: &Option<String>, element: &scraper::ElementRef) -> Value;
}

// The concrete implementation of the folder
pub struct HtmlScraper;

impl Folder for HtmlScraper {
    fn fold_rule(&mut self, rule: &ScrapeRule, element: &scraper::ElementRef) -> Value {
        match rule {
            ScrapeRule::One { selector, name, sub_rules, attribute } => 
                self.fold_one(selector, name, sub_rules, attribute, element),
            ScrapeRule::All { selector, name, sub_rules, attribute } => 
                self.fold_all(selector, name, sub_rules, attribute, element),
            ScrapeRule::Text { selector, name } => 
                self.fold_text(selector, name, element),
        }
    }

    fn fold_one(&mut self, selector: &str, name: &Option<String>, sub_rules: &Option<Vec<ScrapeRule>>, attribute: &Option<String>, element: &scraper::ElementRef) -> Value {
        let selector = Selector::parse(selector).unwrap();
        if let Some(selected_element) = element.select(&selector).next() {
            if let Some(sub_rules) = sub_rules {
                let mut sub_result = Map::new();
                for sub_rule in sub_rules {
                    let sub_value = self.fold_rule(sub_rule, &selected_element);
                    if let Some(name) = sub_rule.name() {
                        sub_result.insert(name.to_string(), sub_value);
                    }
                }
                Value::Object(sub_result)
            } else if let Some(attr) = attribute {
                Value::String(selected_element.value().attr(attr).unwrap_or("").to_string())
            } else {
                Value::String(selected_element.text().collect::<String>().trim().to_string())
            }
        } else {
            Value::Null
        }
    }

    fn fold_all(&mut self, selector: &str, name: &Option<String>, sub_rules: &Option<Vec<ScrapeRule>>, attribute: &Option<String>, element: &scraper::ElementRef) -> Value {
        let selector = Selector::parse(selector).unwrap();
        let selected_elements: Vec<scraper::ElementRef> = element.select(&selector).collect();
        
        let results: Vec<Value> = selected_elements.iter().map(|selected_element| {
            if let Some(sub_rules) = sub_rules {
                let mut sub_result = Map::new();
                for sub_rule in sub_rules {
                    let sub_value = self.fold_rule(sub_rule, selected_element);
                    if let Some(name) = sub_rule.name() {
                        sub_result.insert(name.to_string(), sub_value);
                    }
                }
                Value::Object(sub_result)
            } else if let Some(attr) = attribute {
                Value::String(selected_element.value().attr(attr).unwrap_or("").to_string())
            } else {
                Value::String(selected_element.text().collect::<String>().trim().to_string())
            }
        }).collect();

        Value::Array(results)
    }

    fn fold_text(&mut self, selector: &str, name: &Option<String>, element: &scraper::ElementRef) -> Value {
        let selector = Selector::parse(selector).unwrap();
        let text: String = element.select(&selector)
            .map(|el| el.text().collect::<String>())
            .collect::<Vec<String>>()
            .join(" ")
            .split_whitespace()
            .collect::<Vec<&str>>()
            .join(" ");

        Value::String(text)
    }
}

impl ScrapeRule {
    fn name(&self) -> &Option<String> {
        match self {
            ScrapeRule::One { name, .. } => name,
            ScrapeRule::All { name, .. } => name,
            ScrapeRule::Text { name, .. } => name,
        }
    }
}

impl HtmlScraper {
    pub fn scrape<T: Scrapable>(html: &str) -> Result<Map<String, Value>, Box<dyn std::error::Error>> {
        let document = Html::parse_document(html);
        let rules = T::scrape_rules();
        let mut scraper = HtmlScraper;
        let mut result = Map::new();

        for rule in rules {
            let value = scraper.fold_rule(&rule, &document.root_element());
            if let Some(name) = rule.name() {
                result.insert(name.clone(), value);
            }
        }

        Ok(result)
    }
}

// Usage example
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let html = r#"
        <html>
            <body>
                <h1 class="title">Breaking News</h1>
                <div class="author">John Doe</div>
                <div class="paragraph">This is the first paragraph.</div>
                <div class="paragraph">This is the second paragraph.</div>
            </body>
        </html>
    "#;

    let result = HtmlScraper::scrape::<NewsArticle>(html)?;
    println!("{:#?}", result);

    Ok(())
}