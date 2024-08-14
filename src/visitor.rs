use scraper::{ElementRef, Selector};
use std::{collections::HashMap};

use crate::{cleaner::TextCleaner, scraper_config::ScrapeRule};



// Updated Visitor trait
pub trait Visitor {
    fn visit_element(
        &mut self,
        element: &ElementRef,
        rule: &ScrapeRule,
        cleaner: Option<&dyn TextCleaner>,
    ) ->  HashMap<String, String>;
    fn visit_text(&mut self, text: &str, cleaner: Option<&dyn TextCleaner>) -> String;
}

// Updated concrete visitor
pub struct ScraperVisitor;

impl Visitor for ScraperVisitor {
    fn visit_element(
        &mut self,
        element: &ElementRef,
        rule: &ScrapeRule,
        cleaner: Option<&dyn TextCleaner>,
    ) -> HashMap<String, String> {
        let mut result = HashMap::new();
        match rule {
            ScrapeRule::One {
                selector,
                name,
                sub_rules,
                attribute,
            } => {
                let selector = Selector::parse(selector).unwrap();
                if let Some(selected_element) = element.select(&selector).next() {
                    if let Some(sub_rules) = sub_rules {
                        for sub_rule in sub_rules {
                            result.extend(self.visit_element(&selected_element, sub_rule, cleaner));
                        }
                    } else if let Some(attr) = attribute {
                        let value = selected_element
                            .value()
                            .attr(attr)
                            .unwrap_or("")
                            .to_string();
                        result.insert(name.clone(), self.visit_text(&value, cleaner));
                    } else {
                        let text = selected_element.text().collect::<String>();
                        result.insert(name.clone(), self.visit_text(&text, cleaner));
                    }
                }
            }
            ScrapeRule::All {
                selector,
                name,
                sub_rules,
                attribute,
            } => {
                let selector = Selector::parse(selector).unwrap();
                let selected_elements: Vec<ElementRef> = element.select(&selector).collect();

                let values: Vec<String> = selected_elements
                    .iter()
                    .map(|selected_element| {
                        if let Some(sub_rules) = sub_rules {
                            let mut sub_result = HashMap::new();
                            for sub_rule in sub_rules {
                                sub_result.extend(self.visit_element(
                                    &selected_element,
                                    sub_rule,
                                    cleaner,
                                ));
                            }
                            serde_json::to_string(&sub_result).unwrap()
                        } else if let Some(attr) = attribute {
                            let value = selected_element
                                .value()
                                .attr(attr)
                                .unwrap_or("")
                                .to_string();
                            self.visit_text(&value, cleaner)
                        } else {
                            self.visit_text(&selected_element.text().collect::<String>(), cleaner)
                        }
                    })
                    .collect();

                result.insert(name.clone(), serde_json::to_string(&values).unwrap());
            }
            ScrapeRule::Text { selector, name } => {
                let selector = Selector::parse(selector).unwrap();
                let text: String = element
                    .select(&selector)
                    .map(|el| el.text().collect::<String>())
                    .collect::<Vec<String>>()
                    .join(" ");

                result.insert(name.clone(), self.visit_text(&text, cleaner));
            }
        }
        result
    }

    fn visit_text(&mut self, text: &str, cleaner: Option<&dyn TextCleaner>) -> String {
        if let Some(cleaner) = cleaner {
            cleaner.clean(text)
        } else {
            text.to_string()
        }
    }
}

