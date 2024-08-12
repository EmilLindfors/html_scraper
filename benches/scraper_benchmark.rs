use criterion::{black_box, criterion_group, criterion_main, Criterion};
use html_parser::{HtmlScraper, ScrapeConfig, ScrapeRule, ScraperConfig};
use serde::Deserialize;
use std::collections::HashMap;
use std::borrow::Cow;

#[derive(Debug, Deserialize)]
struct Article {
    title: String,
    author: String,
    content: Vec<String>,
}

impl ScrapeConfig for Article {
    fn get_config() -> ScraperConfig {
        ScraperConfig::new(vec![
                ScrapeRule::One {
                    selector: "h1".to_string(),
                    name: "title".to_string(),
                    sub_rules: None,
                    attribute: None,
                },
                ScrapeRule::One {
                    selector: ".author".to_string(),
                    name: "author".to_string(),
                    sub_rules: None,
                    attribute: None,
                },
                ScrapeRule::All {
                    selector: "p".to_string(),
                    name: "content".to_string(),
                    sub_rules: None,
                    attribute: None,
                },
            ]
        )
    }
}

impl From<HashMap<String, String>> for Article {
   fn from(value: HashMap<String, String>) -> Self {
         let title = value.get("title").unwrap().clone();
         let author = value.get("author").unwrap().clone();
         let content = value.get("content").unwrap().split("\n").map(|s| s.to_string()).collect();
         Article { title, author, content }
   }
}

fn generate_sample_html(paragraphs: usize) -> String {
    let mut html = String::from(r#"
        <html>
        <head><title>Sample Article</title></head>
        <body>
            <h1>Sample Article Title</h1>
            <div class="author">John Doe</div>
    "#);

    for i in 0..paragraphs {
        html.push_str(&format!("<p>This is paragraph {}.</p>\n", i));
    }

    html.push_str("</body></html>");
    html
}

fn bench_scrape(c: &mut Criterion) {
    let html = generate_sample_html(100);  // 100 paragraphs
    let scraper = HtmlScraper::default();

    c.bench_function("scrape 100 paragraphs", |b| {
        b.iter(|| {
            let _article: Article = scraper.scrape(black_box(&html)).unwrap();
        })
    });

    // Benchmark with different numbers of paragraphs
    let paragraph_counts = [10, 50, 100, 500, 1000];
    let mut group = c.benchmark_group("paragraph_scaling");
    for &count in &paragraph_counts {
        let html = generate_sample_html(count);
        group.bench_function(format!("{} paragraphs", count), |b| {
            b.iter(|| {
                let _article: Article = scraper.scrape(black_box(&html)).unwrap();
            })
        });
    }
    group.finish();
}

criterion_group!(benches, bench_scrape);
criterion_main!(benches);