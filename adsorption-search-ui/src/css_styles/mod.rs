use std::fmt::Display;

use css_style::prelude::*;

use self::{
    body::body_style,
    form_elements::{
        all_box_sizing, error_active, error_message, input_error_focus_style, input_error_style,
        input_number_style, input_text_style, label_style,
    },
};

mod body;
mod form_elements;

pub fn build_css_sheet() -> CssSheet {
    let components = vec![
        body_style(),
        label_style(),
        input_text_style(),
        input_number_style(),
        input_error_style(),
        input_error_focus_style(),
        all_box_sizing(),
        error_message(),
        error_active(),
    ];
    CssSheet { components }
}

pub struct ElementStyle {
    name: String,
    style: Style,
}

pub struct ElementStyleBuilder {
    name: Option<String>,
    style: Option<Style>,
}

impl ElementStyleBuilder {
    fn new(name: Option<String>, style: Option<Style>) -> Self {
        Self { name, style }
    }
    fn with_name(self, name: &str) -> Self {
        Self {
            name: Some(name.into()),
            style: self.style,
        }
    }
    fn with_style(self, style: Style) -> Self {
        Self {
            name: self.name,
            style: Some(style),
        }
    }
    fn build(self) -> Result<ElementStyle, BuildError> {
        if self.name.is_some() && self.style.is_some() {
            Ok(ElementStyle {
                name: self.name.unwrap(),
                style: self.style.unwrap(),
            })
        } else {
            Err(BuildError)
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct BuildError;

impl Display for BuildError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Incomplete content supplied to builder!")
    }
}

impl ElementStyle {
    pub fn builder() -> ElementStyleBuilder {
        ElementStyleBuilder::new(None, None)
    }
}

impl Display for ElementStyle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let inline_style = format!("{}", self.style);
        let splitted_text: Vec<String> = inline_style
            .split(";")
            .filter(|text| text.len() > 0)
            .map(|text| format!("  {text};\n"))
            .collect();
        let style = splitted_text.concat();

        write!(f, "{} {{\n{}}}", self.name, style)
    }
}

pub struct CssSheet {
    components: Vec<ElementStyle>,
}

impl Display for CssSheet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let content = self
            .components
            .iter()
            .map(|comp| format!("{}", comp))
            .collect::<Vec<String>>()
            .join("\n");
        write!(f, "{}", content)
    }
}
