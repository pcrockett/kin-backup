use std::io::{ BufReader, Read };
use std::fs::{ File };
use std::path::PathBuf;
use kin_core::templating;
use kin_core::templating::mustache;
use kin_core::{ Error };
use serde::{ Serialize };

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

pub fn render(md_template_path: &PathBuf, model: &ReadmeModel, dest_path: &PathBuf) -> Result<(), Error> {

    let mut md_template_text = String::new();

    {
        let md_file = File::open(md_template_path)?;
        let mut md_file = BufReader::new(md_file);
        md_file.read_to_string(&mut md_template_text)?;
    }

    let md_template = mustache::compile_str(md_template_text.as_str())?;
    let md_content = md_template.render_to_string(model)?;

    templating::render_html(&md_content, &dest_path)
}
