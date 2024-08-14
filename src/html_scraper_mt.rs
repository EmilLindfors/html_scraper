impl HtmlScraper {
    pub fn new() -> HtmlScraperBuilder {
        HtmlScraperBuilder::new()
    }

    pub fn scrape<T: ScrapeConfig + for<'a> From<DashMap<String, String>> + Send>(
        &self,
        html: &str,
    ) -> Result<T, ConfigError> {
        let scraper_config = if let Some(config_str) = &self.config {
            T::from_config(&config_str)?
        } else {
            T::get_config()
        };

        let document = Html::parse_document(html);
        let visitor = Arc::new(ScraperVisitor);
        let thread_safe_cleaner = self.cleaner.as_ref().map(|c| {
            Arc::new(ThreadSafeCleanerWrapper(Arc::new(c.clone()))) as Arc<dyn ThreadSafeTextCleaner>
        });

        let result = DashMap::new();
        scraper_config.rules.par_iter().for_each(|rule| {
            visitor.visit_element(&result, &document.root_element(), rule, thread_safe_cleaner.as_deref());
        });

        Ok(T::from(result))
    }
}