use scraper::{ElementRef, Selector};
use std::collections::HashMap;

use crate::{cleaner::TextCleaner, scraper_config::ScrapeRule};



// Updated Visitor trait
pub trait Visitor {
    fn visit_element(
        &mut self,
        element: &ElementRef,
        rule: &ScrapeRule,
        cleaner: Option<&dyn TextCleaner>,
    ) -> HashMap<String, String>;
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
                                    selected_element,
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




#[cfg(test)]

mod tests {
    use serde::{Deserialize, Serialize};

    use crate::{cleaner::DefaultCleaner, html_scraper::{HtmlScraper, HtmlScraperBuilder}, scraper_config::{ScrapeConfig, ScrapeRule, ScraperConfig}};

    use super::*;

    // NewsArticle struct
#[derive(Debug, Serialize, Deserialize)]
pub struct NewsArticle {
    title: String,
    author: String,
    content: Vec<String>,
}

impl ScrapeConfig for NewsArticle {
    fn get_config() -> ScraperConfig {
        ScraperConfig {
            rules: vec![
                ScrapeRule::One {
                    selector: "h1.title".to_string(),
                    name: "title".to_string(),
                    sub_rules: None,
                    attribute: None,
                },
                ScrapeRule::One {
                    selector: "div.author".to_string(),
                    name: "author".to_string(),
                    sub_rules: None,
                    attribute: None,
                },
                ScrapeRule::All {
                    selector: "div.paragraph".to_string(),
                    name: "content".to_string(),
                    sub_rules: None,
                    attribute: None,
                },
            ],
        }
    }
}

impl From<HashMap<String, String>> for NewsArticle {
    fn from(map: HashMap<String, String>) -> Self {
        NewsArticle {
            title: map.get("title").cloned().unwrap_or_default(),
            author: map.get("author").cloned().unwrap_or_default(),
            content: map
                .get("content")
                .and_then(|s| serde_json::from_str(s).ok())
                .unwrap_or_default(),
        }
    }
}

impl Default for NewsArticle {
    fn default() -> Self {
        NewsArticle {
            title: String::new(),
            author: String::new(),
            content: Vec::new(),
        }
    }
}

    #[test]
    fn test_default() {
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

        // Using default configuration
        let article_default: NewsArticle = HtmlScraper::default().scrape(html).unwrap();
        assert_eq!(article_default.title, "Breaking News");
        assert_eq!(article_default.author, "John Doe");
        assert_eq!(
            article_default.content,
            vec![
                "This is the first paragraph.",
                "This is the second paragraph."
            ]
        );
    }

    #[test]
    fn test_json_config() {
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

        // Using JSON configuration
        let json_config = r#"
    {
        "rules": [
            {
                "type": "One",
                "selector": "h1.title",
                "name": "title"
            },
            {
                "type": "One",
                "selector": "div.author",
                "name": "author"
            },
            {
                "type": "All",
                "selector": "div.paragraph",
                "name": "content"
            }
        ]
    }
    "#;

        let article_json: NewsArticle = HtmlScraperBuilder::new()
            .with_config(json_config)
            .build()
            .scrape(html)
            .unwrap();
        assert_eq!(article_json.title, "Breaking News");
        assert_eq!(article_json.author, "John Doe");
        assert_eq!(
            article_json.content,
            vec![
                "This is the first paragraph.",
                "This is the second paragraph."
            ]
        );
    }

    #[test]
    fn test_read_file() {
        use std::fs;

        let html = fs::read_to_string("./src/tests/data/academic_article.html").unwrap();

        // NewsArticle struct
        #[derive(Debug, Serialize, Deserialize)]
        pub struct Research {
            abstract_: String,
            keywords: Vec<String>,
            introduction: String,
            sub_headings: Vec<String>,
        }

        impl ScrapeConfig for Research {
            fn get_config() -> ScraperConfig {
                ScraperConfig {
                    rules: vec![
                        ScrapeRule::One {
                            selector: "div.hlFld-Abstract".to_string(),
                            name: "abstract_outer".to_string(),
                            sub_rules: Some(vec![ScrapeRule::One {
                                selector: "p.last".to_string(),
                                name: "abstract_".to_string(),
                                sub_rules: None,
                                attribute: None,
                            }]),
                            attribute: None,
                        },
                        ScrapeRule::All {
                            selector: ".abstractKeywords li a".to_string(),
                            name: "keywords".to_string(),
                            sub_rules: None,
                            attribute: None,
                        },
                        ScrapeRule::One {
                            selector: ".NLM_sec_level_1".to_string(),
                            name: "intro_container".to_string(),
                            sub_rules: Some(vec![ScrapeRule::Text {
                                selector: "p".to_string(),
                                name: "introduction".to_string(),
                            }]),
                            attribute: None,
                        },
                        ScrapeRule::All {
                            selector: ".NLM_sec_level_2".to_string(),
                            name: "sub_headings".to_string(),
                            sub_rules: Some(vec![ScrapeRule::One {
                                selector: "h3".to_string(),
                                name: "heading".to_string(),
                                sub_rules: None,
                                attribute: None,
                            }]),
                            attribute: None,
                        },
                    ],
                }
            }
        }

        impl From<HashMap<String, String>> for Research {
            fn from(map: HashMap<String, String>) -> Self {
                println!("{:?}", map);
                Research {
                    abstract_: map.get("abstract_").cloned().unwrap_or_default(),
                    keywords: map
                        .get("keywords")
                        .and_then(|s| serde_json::from_str(s).ok())
                        .unwrap_or_default(),
                    introduction: map.get("introduction").cloned().unwrap_or_default(),
                    sub_headings: map
                        .get("sub_headings")
                        .and_then(|s| serde_json::from_str(s).ok())
                        .unwrap_or_default(),
                }
            }
        }

        let result: Research = HtmlScraperBuilder::new()
            .with_cleaner(DefaultCleaner)
            .build()
            .scrape(&html)
            .unwrap();

        assert_eq!(
            result.abstract_,
            "This paper adds a multidimensional perspective to the study of related diversification. We examine how regions diversify into new jobs – defined as unique industry-occupation combinations – asking whether they do so from related industries or related occupations. We use linked employer-employee data for all labour market regions in Norway, covering the time period 2009–2014. Diversification into new jobs is more likely in the presence of related occupations and industries in a region. Furthermore, occupational and industrial relatedness have complementary effects on diversification. Occupational relatedness and its interaction with industrial relatedness are particularly important for diversification into more complex activities."
        );

        let keywords = result.keywords;

        assert_eq!(keywords.len(), 9);

        assert_eq!(keywords[0].as_str(), "Regional capabilities");

        assert_eq!(keywords[1].as_str(), "jobs");

        assert_eq!(
            result.introduction,
            "Regional economies typically evolve by branching from existing activities into related new activities. While this general insight is widely accepted in evolutionary economic geography, the literature has so far only examined evolution within the same type of activity – e.g., between technologies, industries or occupations. However, economic activities are multidimensional. They involve a person with a specific skill set engaged in an occupation within an industry, typically using different types of technology. This multidimensional perspective has so far been missing from research on related diversification. No research has hitherto examined the relative importance of relatedness across different dimensions for diversification into new types of multidimensional economic activities. This paper is the first to take such a multidimensional perspective. Specifically, we examine diversification into new jobs. A job can be defined as the unique combination of an industry and an occupation (Fernández-Macías, Citation2012; Goos et al., Citation2009; Henning & Eriksson, Citation2021). For example, the occupation of an engineer is significantly shaped by the industry in which it is applied. Consider, for example, the contrast between a biomedical engineer, who designs medical devices, and an aerospace engineer, who develops aircraft and spacecraft systems. Regions specialise in specific jobs, just as they specialise in industries and in occupations. For instance, a region might specialise in aerospace engineering, or in aerospace mechanics, or in car engineering, or in any combination of these. Furthermore, regions can develop the competence to do new jobs by drawing on their capabilities in related industries and/or in related occupations. From the literature, we know that regions are more likely to enter new industries if they are already specialised in related industries (Boschma et al., Citation2013; Essletzbichler, Citation2015; Neffke et al., Citation2011). We also know that regions are more likely to enter new occupations if they are specialised in related occupations (Farinha et al., Citation2019; Muneepeerakul et al., Citation2013). However, we don’t know whether it is industrial or occupational relatedness, or some combination of the two, which matters for entry into new jobs. We also don’t know whether there is any interaction between industrial and occupational relatedness in the diversification process. Put differently, we don’t know whether occupational relatedness can substitute for industrial relatedness, or whether the two are complementary. Finally, we don’t know whether the importance of industrial or occupational relatedness depends on the complexity of the job which the region is diversifying into. To address these questions, we explore how the entry of new regional specialisations at the level of jobs is shaped by the density of related industries and occupations within the region. We also examine how the interaction between industries and occupations shape new job entry, and how the importance of each dimension varies depending on occupational complexity. We use linked employer-employee data from Norway for the period 2009–2014.Footnote1 This is a period of recovery and growth following the financial crisis of 2008 and until the oil price drop in 2014, which affected the Norwegian economy severely due to its reliance on oil exports. The data includes the firm, industry and occupation of individual employees for each year, which we use to track mobility across industries and occupations. From this, we construct skill-relatedness matrices across industries and occupations, using an approach which is well-established in previous empirical research (e.g., Fitjar & Timmermans, Citation2017; Neffke & Henning, Citation2013; Timmermans & Boschma, Citation2014). Furthermore, we study the specialisation of regions in different jobs by measuring regional employment shares and location quotients at the occupation-industry-region level. We find that an increase in industrial relatedness improves the likelihood of regions developing new specialisations at the level of jobs. Occupational relatedness also has a positive, but somewhat weaker, impact on the entry of new job specialisations. However, occupational relatedness matters in particular for diversification into more complex jobs. The two dimensions of relatedness, occupational and industrial relatedness, are complementary insofar as occupational relatedness has a greater impact on the entry of new specialisations when industrial relatedness is high, and vice versa. This complementarity is particularly important for entry into more complex jobs. The remainder of this paper is structured as follows: In the next section, we discuss the diversification of regions into new economic activities from an evolutionary economic geography perspective. In the third section, we introduce the Norwegian case and explain how we measure occupational and industrial relatedness. In the fourth section, we describe the empirical approach to studying the impact of occupational and industrial relatedness on diversification into new jobs. In the fifth section, we empirically study the diversification of Norwegian regions into new jobs. The final section concludes and discusses policy implications."
        );

        let sub_headings = result.sub_headings;

        assert_eq!(sub_headings.len(), 5);
    }

    #[test]
    fn test_read_file2() {
        use std::fs;

        let html = fs::read_to_string("./src/tests/data/ilaks_news.html").unwrap();

        // NewsArticle struct
        #[derive(Debug, Serialize, Deserialize)]
        pub struct News {
            paragraph: Vec<String>,
        }

        impl ScrapeConfig for News {
            fn get_config() -> ScraperConfig {
                ScraperConfig {
                    rules: vec![ScrapeRule::One {
                        selector: ".td-post-content".to_string(),
                        name: "content".to_string(),
                        sub_rules: Some(vec![ScrapeRule::All {
                            selector: "p".to_string(),
                            name: "paragraph".to_string(),
                            sub_rules: None,
                            attribute: None,
                        }]),
                        attribute: None,
                    }],
                }
            }
        }

        impl From<HashMap<String, String>> for News {
            fn from(map: HashMap<String, String>) -> Self {
                News {
                    paragraph: map
                        .get("paragraph")
                        .and_then(|s| serde_json::from_str(s).ok())
                        .unwrap_or_default(),
                }
            }
        }

        let result: News = HtmlScraperBuilder::new()
            .with_cleaner(DefaultCleaner)
            .build()
            .scrape(&html)
            .unwrap();
        assert_eq!(
            result.paragraph.last().unwrap(),
"— Dette vil være et sårt tiltrengt tilskudd langs kyststripen for å skape interesse for en av landets aller største næringer. Fremtidig kompetanse begynner med barn og unges nysgjerrighet. Vi har derfor tro på at 250m2 med prøve-selv- installasjoner, spill og konkurranser vil kunne bidra til at flere får øynene opp for mulighetene som ligger i hav og sjømat, sier leder for Vitensenteret Sørlandet, Kine Wangerud." );
    }

    #[test]
    fn test_config_struct() {
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

        let config = ScraperConfig {
            rules: vec![
                ScrapeRule::One {
                    selector: "h1.title".to_string(),
                    name: "title".to_string(),
                    sub_rules: None,
                    attribute: None,
                },
                ScrapeRule::One {
                    selector: "div.author".to_string(),
                    name: "author".to_string(),
                    sub_rules: None,
                    attribute: None,
                },
                ScrapeRule::All {
                    selector: "div.paragraph".to_string(),
                    name: "content".to_string(),
                    sub_rules: None,
                    attribute: None,
                },
            ],
        };

        let article: NewsArticle = HtmlScraperBuilder::new()
            .with_config(&config.to_string())
            .build()
            .scrape(html)
            .unwrap();

        assert_eq!(article.title, "Breaking News");
        assert_eq!(article.author, "John Doe");
        assert_eq!(
            article.content,
            vec![
                "This is the first paragraph.",
                "This is the second paragraph."
            ]
        );
    }
}
