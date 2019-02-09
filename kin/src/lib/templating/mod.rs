use pulldown_cmark::{ Parser, html };
use std::fs::{ File };
use std::io::{ BufWriter, Write };
use std::path::PathBuf;

pub fn render_html(markdown_content: &String, dest_path: &PathBuf) -> Result<(), failure::Error> {

    let parser = Parser::new(markdown_content.as_str());
    let mut html_content = String::new();
    html::push_html(&mut html_content, parser);

    let html_file = File::create(&dest_path)?;
    let mut html_file = BufWriter::new(html_file);
    html_file.write_all(html_content.as_bytes())?;

    Ok(())
}
