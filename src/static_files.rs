pub mod static_files {
    use std::{
        fmt::Debug,
        fs::File,
        hash::Hash,
        io::{Error, Read},
    };

    use reqwest::Url;

    use crate::{
        engines::engine_base::engine_base::{SearchEngine, SearchResult},
        utils::utils::hash_string,
    };

    pub fn read_file_contents(path: &str) -> Result<String, Error> {
        let mut contents = String::new();

        let mut file = File::open(path)?;

        file.read_to_string(&mut contents)?;

        Ok(contents)
    }

    const HTML_BEGINNING: &str = include_str!("./public/html/beginning.html");
    const HTML_BEGINNING_QUERY_REPLACE: &str = r#"{% search_value %}"#;

    pub fn render_beginning_html(query: &str) -> String {
        HTML_BEGINNING.replace(
            &HTML_BEGINNING_QUERY_REPLACE,
            &html_escape::encode_quoted_attribute(query),
        )
    }

    const FINISHED_CSS: &str = include_str!("./public/css/finished.css");

    pub fn render_finished_css(engine: &str, time: i128) -> String {
        format!(
            "<style>{}</style>",
            FINISHED_CSS
                .replace("__engine__", engine)
                .replace("{% time %}", &time.to_string())
        )
    }

    const HTML_RESULT: &str = include_str!("./public/html/result.html");

    pub fn render_result(result: &SearchResult) -> String {
        HTML_RESULT
            .replace("{% title %}", &result.title)
            .replace("{% url %}", &result.url)
            .replace(
                "{% url_host %}",
                Url::parse(&result.url).unwrap().host_str().unwrap(),
            )
            .replace("{% description %}", &result.description)
            .replace("__ID__", &result.get_html_id())
            .replace(
                "{% image_url %}",
                &result.image_url.clone().unwrap_or("".to_string()),
            )
            .replace(
                "{% date %}",
                &(match &result.date {
                    Some(date_info) => date_info.date.format("%d. %B %Y").to_string(),
                    None => "".to_string(),
                }),
            )
    }

    pub fn render_result_engine_visibility(id: &str, engine: &SearchEngine) -> String {
        format!(
            "<style>#{} .search-engines .{} {{ opacity: 1 !important; }}</style>",
            id,
            engine.to_string().to_lowercase()
        )
    }
}
