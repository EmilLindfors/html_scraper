use scraper::{ElementRef, Html, Selector};
use serde_json::{Map, Value};
pub mod cleaner;
pub mod visitor;

pub struct HtmlScraper;

#[derive(Clone)]
pub enum Extract {
    One {
        locator: String,
        name: Option<String>,
        sub_entities: Option<Vec<Extract>>,
        attribute: Option<String>,
    },
    All {
        locator: String,
        name: Option<String>,
        sub_entities: Option<Vec<Extract>>,
        attribute: Option<String>,
    },
    Text {
        locator: String,
        name: Option<String>,
    },
}

impl HtmlScraper {
    pub fn scrape_html(
        html: &str,
        steps: Vec<Extract>,
    ) -> Result<Map<String, Value>, ScraperError> {
        let document = Html::parse_document(html);
        let mut result = Map::new();

        for step in steps {
            Self::extract(&document, &mut result, step)?;
        }

        Ok(result)
    }

    fn extract(
        document: &Html,
        result: &mut Map<String, Value>,
        extract: Extract,
    ) -> Result<(), ScraperError> {
        match extract {
            Extract::One {
                locator,
                name,
                sub_entities,
                attribute,
            } => {
                Self::extract_one(document, result, &locator, name, sub_entities, attribute)?;
            }
            Extract::All {
                locator,
                name,
                sub_entities,
                attribute,
            } => {
                Self::extract_all(document, result, &locator, name, sub_entities, attribute)?;
            }
            Extract::Text { locator, name } => {
                Self::extract_text(document, result, &locator, name)?;
            }
        }
        Ok(())
    }

    fn extract_one(
        document: &Html,
        result: &mut Map<String, Value>,
        locator: &str,
        name: Option<String>,
        sub_entities: Option<Vec<Extract>>,
        attribute: Option<String>,
    ) -> Result<(), ScraperError> {
        let selector = Selector::parse(locator)
            .map_err(|_| ScraperError::InvalidSelector(locator.to_string()))?;
        let element = document
            .select(&selector)
            .next()
            .ok_or(ScraperError::ElementNotFound(locator.to_string()))?;

        let value = if let Some(sub_entities) = sub_entities {
            let mut sub_result = Map::new();
            for sub_entity in sub_entities {
                Self::extract(
                    &Html::parse_fragment(&element.html()),
                    &mut sub_result,
                    sub_entity,
                )?;
            }
            Value::Object(sub_result)
        } else if let Some(attr) = attribute {
            Value::String(element.value().attr(&attr).unwrap_or("").to_string())
        } else {
            Value::String(element.text().collect::<String>().trim().to_string())
        };

        result.insert(name.unwrap_or_else(|| locator.to_string()), value);
        Ok(())
    }

    fn extract_all(
        document: &Html,
        result: &mut Map<String, Value>,
        locator: &str,
        name: Option<String>,
        sub_entities: Option<Vec<Extract>>,
        attribute: Option<String>,
    ) -> Result<(), ScraperError> {
        let selector = Selector::parse(locator)
            .map_err(|_| ScraperError::InvalidSelector(locator.to_string()))?;
        let elements: Vec<ElementRef> = document.select(&selector).collect();

        if elements.is_empty() {
            return Err(ScraperError::ElementNotFound(locator.to_string()));
        }

        let mut all_results = Vec::new();

        for element in elements {
            let value = if let Some(sub_entities) = sub_entities.as_ref() {
                let mut sub_result = Map::new();
                for sub_entity in sub_entities {
                    Self::extract(
                        &Html::parse_fragment(&element.html()),
                        &mut sub_result,
                        sub_entity.clone(),
                    )?;
                }
                Value::Object(sub_result)
            } else if let Some(attr) = attribute.clone() {
                Value::String(element.value().attr(&attr).unwrap_or("").to_string())
            } else {
                Value::String(element.text().collect::<String>().trim().to_string())
            };
            all_results.push(value);
        }

        result.insert(
            name.unwrap_or_else(|| locator.to_string()),
            Value::Array(all_results),
        );
        Ok(())
    }

    fn extract_text(
        document: &Html,
        result: &mut Map<String, Value>,
        locator: &str,
        name: Option<String>,
    ) -> Result<(), ScraperError> {
        let selector = Selector::parse(locator)
            .map_err(|_| ScraperError::InvalidSelector(locator.to_string()))?;
        let text: String = document
            .select(&selector)
            .map(|element| element.text().collect::<String>())
            .collect::<Vec<String>>()
            .join(" ")
            .split_whitespace()
            .collect::<Vec<&str>>()
            .join(" ");

        result.insert(
            name.unwrap_or_else(|| locator.to_string()),
            Value::String(text),
        );
        Ok(())
    }
}

#[derive(Debug, PartialEq)]
pub enum ScraperError {
    InvalidSelector(String),
    ElementNotFound(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_attribute_extraction() {
        let html = r#"
        <div class="container">
            <a href="https://example.com" class="link">Link</a>
        </div>
        "#;

        let steps = vec![Extract::One {
            locator: ".link".to_string(),
            name: Some("link".to_string()),
            sub_entities: None,
            attribute: Some("href".to_string()),
        }];

        let result = HtmlScraper::scrape_html(html, steps).unwrap();

        println!("{:#?}", result);

        assert_eq!(
            result.get("link").unwrap().as_str().unwrap(),
            "https://example.com"
        );
    }

    #[test]
    fn test_scrape_academic_article() {
        let html = r#"
        <div class="hlFld-Abstract">
            <h2 class="section-heading-2" id="abstract">ABSTRACT</h2>
            <p class="last">This is the abstract text.</p>
        </div>
        <div class="abstractKeywords">
            <div class="hlFld-KeywordText">
                <div>
                    <p id="kwd-title" class="kwd-title">KEYWORDS: </p>
                    <ul>
                        <li><a href="/keyword/Term1" class="kwd-btn keyword-click">Term1</a></li>
                        <li><a href="/keyword/Term2" class="kwd-btn keyword-click">Term2</a></li>
                    </ul>
                </div>
            </div>
        </div>
        <div class="NLM_sec NLM_sec_level_1" id="S001">
            <h2 class="section-heading-2" id="introduction">1. INTRODUCTION</h2>
            <p>This is the introduction text.</p>
        </div>
        "#;

        let steps = vec![
            Extract::One {
                locator: ".hlFld-Abstract".to_string(),
                name: Some("abstract".to_string()),
                sub_entities: Some(vec![Extract::Text {
                    locator: "p.last".to_string(),
                    name: Some("text".to_string()),
                }]),
                attribute: None,
            },
            Extract::All {
                locator: ".abstractKeywords li a".to_string(),
                name: Some("keywords".to_string()),
                sub_entities: None,
                attribute: None,
            },
            Extract::One {
                locator: ".NLM_sec_level_1".to_string(),
                name: Some("introduction".to_string()),
                sub_entities: Some(vec![Extract::Text {
                    locator: "p".to_string(),
                    name: Some("text".to_string()),
                }]),
                attribute: None,
            },
        ];

        let result = HtmlScraper::scrape_html(html, steps).unwrap();

        println!("{:#?}", result);

        assert_eq!(
            result
                .get("abstract")
                .unwrap()
                .get("text")
                .unwrap()
                .as_str()
                .unwrap(),
            "This is the abstract text."
        );

        let keywords = result.get("keywords").unwrap().as_array().unwrap();
        assert_eq!(keywords.len(), 2);
        assert_eq!(keywords[0].as_str().unwrap(), "Term1");
        assert_eq!(keywords[1].as_str().unwrap(), "Term2");

        assert_eq!(
            result
                .get("introduction")
                .unwrap()
                .get("text")
                .unwrap()
                .as_str()
                .unwrap(),
            "This is the introduction text."
        );
    }

    #[test]
    fn test_scrape_google_search_results() {
        let html = r#"
        <div id="search">
            <div class="g">
                <h3 class="LC20lb">Title 1</h3>
                <div class="TbwUpd NJjxre">
                    <cite class="iUh30">www.example.com</cite>
                </div>
                <span class="st">Description 1</span>
            </div>
            <div class="g">
                <h3 class="LC20lb">Title 2</h3>
                <div class="TbwUpd NJjxre">
                    <cite class="iUh30">www.example.com</cite>
                </div>
                <span class="st">Description 2</span>
            </div>
        </div>
        "#;

        let steps = vec![Extract::All {
            locator: ".g".to_string(),
            name: None,
            sub_entities: Some(vec![
                Extract::Text {
                    locator: "h3".to_string(),
                    name: Some("title".to_string()),
                },
                Extract::Text {
                    locator: "cite".to_string(),
                    name: Some("url".to_string()),
                },
                Extract::Text {
                    locator: "span.st".to_string(),
                    name: Some("description".to_string()),
                },
            ]),
            attribute: None,
        }];

        let result = HtmlScraper::scrape_html(html, steps).unwrap();

        println!("{:#?}", result);

        let results = result.get(".g").unwrap().as_array().unwrap();
        assert_eq!(results.len(), 2);

        let result1 = &results[0];
        assert_eq!(result1.get("title").unwrap().as_str().unwrap(), "Title 1");
        assert_eq!(
            result1.get("url").unwrap().as_str().unwrap(),
            "www.example.com"
        );
        assert_eq!(
            result1.get("description").unwrap().as_str().unwrap(),
            "Description 1"
        );

        let result2 = &results[1];
        assert_eq!(result2.get("title").unwrap().as_str().unwrap(), "Title 2");
        assert_eq!(
            result2.get("url").unwrap().as_str().unwrap(),
            "www.example.com"
        );
        assert_eq!(
            result2.get("description").unwrap().as_str().unwrap(),
            "Description 2"
        );
    }

    #[test]
    fn test_invalid_selector() {
        let html = r#"<div></div>"#;

        let steps = vec![Extract::One {
            locator: "invalid".to_string(),
            name: None,
            sub_entities: None,
            attribute: None,
        }];

        let result = HtmlScraper::scrape_html(html, steps);

        assert_eq!(
            result,
            Err(ScraperError::ElementNotFound("invalid".to_string()))
        );
    }

    #[test]
    fn test_element_not_found() {
        let html = r#"<div></div>"#;

        let steps = vec![Extract::One {
            locator: "div.invalid".to_string(),
            name: None,
            sub_entities: None,
            attribute: None,
        }];

        let result = HtmlScraper::scrape_html(html, steps);

        assert_eq!(
            result,
            Err(ScraperError::ElementNotFound("div.invalid".to_string()))
        );
    }

    #[test]
    fn test_extract_text() {
        let html = r#"
        <div>
            <p>Text 1</p>
            <p>Text 2</p>
        </div>
        "#;

        let steps = vec![Extract::Text {
            locator: "p".to_string(),
            name: None,
        }];

        let result = HtmlScraper::scrape_html(html, steps).unwrap();

        println!("{:#?}", result);

        let text = result.get("p").unwrap().as_str().unwrap();
        assert_eq!(text, "Text 1 Text 2");
    }

    #[test]
    fn read_html_file() {
        use std::fs;

        let html = fs::read_to_string("./src/tests/data/academic_article.html").unwrap();

        let steps = vec![
            Extract::One {
                locator: ".hlFld-Abstract".to_string(),
                name: Some("abstract".to_string()),
                sub_entities: Some(vec![Extract::Text {
                    locator: "p.last".to_string(),
                    name: Some("text".to_string()),
                }]),
                attribute: None,
            },
            Extract::All {
                locator: ".abstractKeywords li a".to_string(),
                name: Some("keywords".to_string()),
                sub_entities: None,
                attribute: None,
            },
            Extract::One {
                locator: ".NLM_sec_level_1".to_string(),
                name: Some("introduction".to_string()),
                sub_entities: Some(vec![Extract::Text {
                    locator: "p".to_string(),
                    name: Some("text".to_string()),
                }]),
                attribute: None,
            },
            Extract::All {
                locator: ".NLM_sec_level_2".to_string(),
                name: Some("sub_headings".to_string()),
                sub_entities: Some(vec![Extract::One {
                    locator: "h3".to_string(),
                    name: Some("heading".to_string()),
                    sub_entities: None,
                    attribute: None,
                }]),
                attribute: None,
            },
        ];
        let result = HtmlScraper::scrape_html(&html, steps).unwrap();

        assert_eq!(
            result
                .get("abstract")
                .unwrap()
                .get("text")
                .unwrap()
                .as_str()
                .unwrap(),
            "This paper adds a multidimensional perspective to the study of related diversification. We examine how regions diversify into new jobs – defined as unique industry-occupation combinations – asking whether they do so from related industries or related occupations. We use linked employer-employee data for all labour market regions in Norway, covering the time period 2009–2014. Diversification into new jobs is more likely in the presence of related occupations and industries in a region. Furthermore, occupational and industrial relatedness have complementary effects on diversification. Occupational relatedness and its interaction with industrial relatedness are particularly important for diversification into more complex activities."
        );

        let keywords = result.get("keywords").unwrap().as_array().unwrap();

        assert_eq!(keywords.len(), 9);

        assert_eq!(keywords[0].as_str().unwrap(), "Regional capabilities");

        assert_eq!(keywords[1].as_str().unwrap(), "jobs");

        assert_eq!(
            result
                .get("introduction")
                .unwrap()
                .get("text")
                .unwrap()
                .as_str()
                .unwrap(),
            "Regional economies typically evolve by branching from existing activities into related new activities. While this general insight is widely accepted in evolutionary economic geography, the literature has so far only examined evolution within the same type of activity – e.g., between technologies, industries or occupations. However, economic activities are multidimensional. They involve a person with a specific skill set engaged in an occupation within an industry, typically using different types of technology. This multidimensional perspective has so far been missing from research on related diversification. No research has hitherto examined the relative importance of relatedness across different dimensions for diversification into new types of multidimensional economic activities. This paper is the first to take such a multidimensional perspective. Specifically, we examine diversification into new jobs. A job can be defined as the unique combination of an industry and an occupation (Fernández-Macías, Citation2012; Goos et al., Citation2009; Henning & Eriksson, Citation2021). For example, the occupation of an engineer is significantly shaped by the industry in which it is applied. Consider, for example, the contrast between a biomedical engineer, who designs medical devices, and an aerospace engineer, who develops aircraft and spacecraft systems. Regions specialise in specific jobs, just as they specialise in industries and in occupations. For instance, a region might specialise in aerospace engineering, or in aerospace mechanics, or in car engineering, or in any combination of these. Furthermore, regions can develop the competence to do new jobs by drawing on their capabilities in related industries and/or in related occupations. From the literature, we know that regions are more likely to enter new industries if they are already specialised in related industries (Boschma et al., Citation2013; Essletzbichler, Citation2015; Neffke et al., Citation2011). We also know that regions are more likely to enter new occupations if they are specialised in related occupations (Farinha et al., Citation2019; Muneepeerakul et al., Citation2013). However, we don’t know whether it is industrial or occupational relatedness, or some combination of the two, which matters for entry into new jobs. We also don’t know whether there is any interaction between industrial and occupational relatedness in the diversification process. Put differently, we don’t know whether occupational relatedness can substitute for industrial relatedness, or whether the two are complementary. Finally, we don’t know whether the importance of industrial or occupational relatedness depends on the complexity of the job which the region is diversifying into. To address these questions, we explore how the entry of new regional specialisations at the level of jobs is shaped by the density of related industries and occupations within the region. We also examine how the interaction between industries and occupations shape new job entry, and how the importance of each dimension varies depending on occupational complexity. We use linked employer-employee data from Norway for the period 2009–2014.Footnote1 This is a period of recovery and growth following the financial crisis of 2008 and until the oil price drop in 2014, which affected the Norwegian economy severely due to its reliance on oil exports. The data includes the firm, industry and occupation of individual employees for each year, which we use to track mobility across industries and occupations. From this, we construct skill-relatedness matrices across industries and occupations, using an approach which is well-established in previous empirical research (e.g., Fitjar & Timmermans, Citation2017; Neffke & Henning, Citation2013; Timmermans & Boschma, Citation2014). Furthermore, we study the specialisation of regions in different jobs by measuring regional employment shares and location quotients at the occupation-industry-region level. We find that an increase in industrial relatedness improves the likelihood of regions developing new specialisations at the level of jobs. Occupational relatedness also has a positive, but somewhat weaker, impact on the entry of new job specialisations. However, occupational relatedness matters in particular for diversification into more complex jobs. The two dimensions of relatedness, occupational and industrial relatedness, are complementary insofar as occupational relatedness has a greater impact on the entry of new specialisations when industrial relatedness is high, and vice versa. This complementarity is particularly important for entry into more complex jobs. The remainder of this paper is structured as follows: In the next section, we discuss the diversification of regions into new economic activities from an evolutionary economic geography perspective. In the third section, we introduce the Norwegian case and explain how we measure occupational and industrial relatedness. In the fourth section, we describe the empirical approach to studying the impact of occupational and industrial relatedness on diversification into new jobs. In the fifth section, we empirically study the diversification of Norwegian regions into new jobs. The final section concludes and discusses policy implications."
        );

        let sub_headings = result.get("sub_headings").unwrap().as_array().unwrap();

        assert_eq!(sub_headings.len(), 5);
    }

    #[test]
    fn read_html_file2() {
        use std::fs;

        let html = fs::read_to_string("./src/tests/data/ilaks_news.html").unwrap();

        let steps = vec![Extract::One {
            locator: ".td-post-content".to_string(),
            name: Some("content".to_string()),
            sub_entities: Some(vec![Extract::All {
                locator: "p".to_string(),
                name: Some("paragraph".to_string()),
                sub_entities: None,
                attribute: None,
            }]),
            attribute: None,
        }];

        let result = HtmlScraper::scrape_html(&html, steps).unwrap();

        // check the last paragraph
        let paragraphs = result
            .get("content")
            .unwrap()
            .get("paragraph")
            .unwrap()
            .as_array()
            .unwrap();
        let last_paragraph = paragraphs.last().unwrap();

        assert_eq!(
            last_paragraph.to_string(),
"— Dette vil være et sårt tiltrengt tilskudd langs kyststripen for å skape interesse for en av
                            landets aller største næringer. Fremtidig kompetanse begynner med barn og unges
                            nysgjerrighet. Vi har derfor tro på at 250m2 med prøve-selv- installasjoner, spill og
                            konkurranser vil kunne bidra til at flere får øynene opp for mulighetene som ligger i hav og
                            sjømat, sier leder for Vitensenteret Sørlandet, Kine Wangerud." );
    }
}
