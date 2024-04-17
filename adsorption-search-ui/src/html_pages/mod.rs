use html::metadata::builders::HeadBuilder;

pub mod root_page;
pub mod upload_response_page;

pub fn head(x: &mut HeadBuilder) -> &mut HeadBuilder {
    x.title(|t| t.title("Adsorption Site Searcher"))
        .meta(|meta| {
            meta.name("viewport")
                .content("width=device-width, initial-scale=1")
                .charset("utf-8")
        })
        .link(|link| link.rel("stylesheet").href("/new_style.css"))
}
