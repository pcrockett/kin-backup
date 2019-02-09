use mustache;
use mustache::Template;
use pulldown_cmark::{ Parser, html };
use std::fs::{ File };
use std::io::{ BufReader, BufWriter, Read };
use std::path::PathBuf;

#[derive(Serialize)]
pub struct ReadmeModel {
    pub owner: String,
    pub recipient: String,
    pub passphrase: String,
    pub peers: Vec<PeerModel>
}

#[derive(Serialize)]
pub struct PeerModel {
    pub name: String
}

#[derive(Serialize)]
struct HtmlModel {
    body: String
}

pub fn render_readme(md_template_path: &PathBuf, model: &ReadmeModel, dest_path: &PathBuf) -> Result<(), failure::Error> {

    let mut md_template_text = String::new();

    {
        let md_file = File::open(md_template_path)?;
        let mut md_file = BufReader::new(md_file);
        md_file.read_to_string(&mut md_template_text)?;
    }

    let md_template = mustache::compile_str(md_template_text.as_str())?;
    let md_content = md_template.render_to_string(model)?;

    render_html(&md_content, &dest_path)
}

pub fn render_html(markdown_content: &String, dest_path: &PathBuf) -> Result<(), failure::Error> {

    let parser = Parser::new(markdown_content.as_str());
    let mut html_body = String::new();
    html::push_html(&mut html_body, parser);

    let html_template = get_html_template()?;
    let html_model = HtmlModel {
        body: html_body
    };

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