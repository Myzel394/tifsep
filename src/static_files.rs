pub mod static_files {
    use lazy_static::lazy_static;
    use std::{
        fs::File,
        io::{Error, Read},
    };

    pub fn read_file_contents(path: &str) -> Result<String, Error> {
        let mut contents = String::new();

        let mut file = File::open(path)?;

        file.read_to_string(&mut contents)?;

        Ok(contents)
    }

    lazy_static! {
        static ref HTML_BEGINNING: String =
            read_file_contents("./src/public/html/beginning.html").unwrap();
    }

    const HTML_BEGINNING_QUERY_REPLACE: &str = r#"{% search_value %}"#;

    pub fn render_beginning_html(query: &str) -> String {
        HTML_BEGINNING.replace(
            &HTML_BEGINNING_QUERY_REPLACE,
            &html_escape::encode_quoted_attribute(query),
        )
    }

    lazy_static! {
        static ref FINISHED_CSS: String =
            read_file_contents("./src/public/css/finished.css").unwrap();
    }

    pub fn render_finished_css(engine: &str, time: i128) -> String {
        format!(
            "<style>{}</style>",
            FINISHED_CSS
                .replace("__engine__", engine)
                .replace("{% time %}", &format!("{}ms", &time.to_string()))
        )
    }
}
