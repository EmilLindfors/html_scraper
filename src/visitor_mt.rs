use scraper::{Html, Selector};
use rayon::prelude::*;
use std::collections::HashMap;
use std::sync::Arc;

pub struct HtmlParser {
    rules: Vec<ParseRule>,
}

pub enum ParseRule {
    Single {
        name: String,
        selector: String,
    },
    Multiple {
        name: String,
        selector: String,
    },
}

impl HtmlParser {
    pub fn new(rules: Vec<ParseRule>) -> Self {
        HtmlParser { rules }
    }

    pub fn parse(&self, html: &str) -> HashMap<String, Vec<String>> {
        let document = Arc::new(Html::parse_document(html));
        
        self.rules.par_iter()
            .map(|rule| {
                let result = match rule {
                    ParseRule::Single { name, selector } => {
                        let selector = Selector::parse(selector).unwrap();
                        let text = document.select(&selector)
                            .next()
                            .map(|element| element.text().collect::<String>())
                            .unwrap_or_default();
                        (name.clone(), vec![text])
                    },
                    ParseRule::Multiple { name, selector } => {
                        let selector = Selector::parse(selector).unwrap();
                        let texts: Vec<String> = document.select(&selector)
                            .map(|element| element.text().collect())
                            .collect();
                        (name.clone(), texts)
                    },
                };
                result
            })
            .collect()
    }
}

// Example usage
fn main() {
    let html = r#"
        <html>
            <body>
                <h1>Hello, world!</h1>
                <p class="content">First paragraph</p>
                <p class="content">Second paragraph</p>
            </body>
        </html>
    "#;

    let rules = vec![
        ParseRule::Single {
            name: "title".to_string(),
            selector: "h1".to_string(),
        },
        ParseRule::Multiple {
            name: "paragraphs".to_string(),
            selector: "p.content".to_string(),
        },
    ];

    let parser = HtmlParser::new(rules);
    let result = parser.parse(html);

    println!("{:?}", result);
}