#![recursion_limit = "512"]

pub use css_styles::build_css_sheet;
pub use html_pages::{build_root_page, build_upload_response_page};

mod css_styles;
mod html_pages;

// fn main() -> Result<(), std::io::Error> {
//     let page = build_root_page();
//     let export = format!("{}", page);
//     fs::write("index_new.html", export)?;
//     let css_sheet = build_css_sheet();
//     fs::write("assets/new_style.css", format!("{}", css_sheet))
// }
