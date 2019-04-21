pub use mustache;
use mustache::Template;
use pulldown_cmark::{html, Parser};
use serde::Serialize;
use std::fs::File;
use std::io::BufWriter;
use std::path::PathBuf;

#[derive(Serialize)]
struct HtmlModel {
    body: String,
}

pub fn render_html(markdown_content: &String, dest_path: &PathBuf) -> Result<(), failure::Error> {
    let parser = Parser::new(markdown_content.as_str());
    let mut html_body = String::new();
    html::push_html(&mut html_body, parser);

    let html_template = get_html_template()?;
    let html_model = HtmlModel { body: html_body };

    let html_file = File::create(&dest_path)?;
    let mut html_file = BufWriter::new(html_file);
    html_template.render(&mut html_file, &html_model)?;

    Ok(())
}

fn get_html_template() -> Result<Template, failure::Error> {
    let template_text = include_str!("template.html");
    let template = mustache::compile_str(template_text)?;
    Ok(template)
}
