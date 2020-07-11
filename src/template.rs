use liquid::{Object, Template};

use crate::mime::{Mime, MimeAware};

use std::collections::HashMap;
use std::error::Error;

pub struct State {
    templates: TemplateMap,
}

impl State {
    pub fn new(templates: TemplateMap) -> Self {
        State {
            templates,
        }
    }
}

pub type TemplateMap = HashMap<String, Template>;

#[derive(Debug, thiserror::Error)]
pub enum TemplateError {
    #[error("Invalid template path: `{0}`")]
    InvalidTemplatePath(String),
    #[error("Template not found: `{0}`")]
    TemplateNotFound(String),
}

pub async fn compile_templates(paths: &[&str]) -> Result<TemplateMap, Box<dyn Error>> {
    let compiler = liquid::ParserBuilder::with_stdlib().build()?;

    let mut map = TemplateMap::new();
    for path in paths {
        let name = path
            .split("/")
            .last()
            .map(|name| name.trim_end_matches(".liquid"))
            .ok_or_else(|| TemplateError::InvalidTemplatePath(path.to_string()))?;
        let source = tokio::fs::read_to_string(path).await?;
        let template = compiler.parse(&source)?;
        map.insert(name.to_string(), template);
    }

    Ok(map)
}

pub async fn serve_template(state: &State, name: &str, mime: Mime) -> Result<impl warp::Reply, Box<dyn Error>> {
    let template = state.templates.get(name).ok_or_else(|| TemplateError::TemplateNotFound(name.to_string()))?;
    let globals: Object = Default::default();
    let markup = template.render(&globals)?;

    Ok(http::Response::builder()
    .content_type(mime)
    .body(markup))
}
