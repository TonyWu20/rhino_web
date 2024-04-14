use html::metadata::builders::HeadBuilder;

mod root_page;
mod upload_response_page;

pub use root_page::build_root_page;
pub use upload_response_page::build_upload_response_page;

pub fn head(x: &mut HeadBuilder) -> &mut HeadBuilder {
    x.title(|t| t.title("Adsorption Site Searcher"))
        .meta(|meta| {
            meta.name("viewport")
                .content("width=device-width, initial-scale=1")
                .charset("utf-8")
        })
        .link(|link| link.rel("stylesheet").href("/assets/new_style.css"))
}
