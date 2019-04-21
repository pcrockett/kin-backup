use kin_core::templating;
use kin_core::templating::mustache;
use kin_core::{bail, Error};
use serde::Serialize;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::PathBuf;

#[derive(Serialize)]
pub struct ReadmeModel {
    pub owner: String,
    pub recipient: String,
    pub passphrase: String,
    pub peers: Vec<PeerModel>,
}

#[derive(Serialize)]
pub struct PeerModel {
    pub name: String,
}

pub fn render(
    md_template_path: &PathBuf,
    model: &ReadmeModel,
    dest_path: &PathBuf,
) -> Result<(), Error> {
    let mut md_template_text = String::new();

    {
        let md_file = match File::open(md_template_path) {
            Ok(file) => file,
            Err(e) => bail!(
                "unable to open template file {}: {}",
                md_template_path.to_str().unwrap(),
                e
            ),
        };

        let mut md_file = BufReader::new(md_file);
        let result = md_file.read_to_string(&mut md_template_text);

        if result.is_err() {
            bail!(
                "unable to read template file {}: {}",
                md_template_path.to_str().unwrap(),
                result.err().unwrap()
            );
        }
    }

    let md_template = match mustache::compile_str(md_template_text.as_str()) {
        Ok(template) => template,
        Err(e) => bail!("unable to compile mustache template: {}", e),
    };

    let md_content = match md_template.render_to_string(model) {
        Ok(content) => content,
        Err(e) => bail!("unable to render mustache template: {}", e),
    };

    templating::render_html(&md_content, &dest_path)
}
