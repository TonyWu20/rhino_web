use html::root::{builders::BodyBuilder, Html};

use super::head;

pub fn build_upload_response_page(text: String) -> Html {
    Html::builder()
        .head(head)
        .body(|b| upload_page_body(b, text))
        .build()
}

fn upload_page_body(body: &mut BodyBuilder, text: String) -> &mut BodyBuilder {
    body.header(|h| h.heading_1(|h1| h1.text("Job submitted!")))
        .header(|h| h.heading_2(|h2| h2.text("Submitted parameters:")))
        .paragraph(|p| p.text(text))
        .anchor(|anchor| anchor.href("/").button(|b| b.text("Back")))
}

fn error_symbol_page_body(body: &mut BodyBuilder, symbol_input: String) -> &mut BodyBuilder {
    body.header(|h| h.heading_1(|h1| h1.text("Error: Invalid input of Element Symbol!")))
        .paragraph(|p| p.text(format!("Your input is: {}", symbol_input)))
        .anchor(|anchor| anchor.href("/").button(|b| b.text("Back")))
}

pub fn build_error_symbol_page(symbol_input: String) -> Html {
    Html::builder()
        .head(head)
        .body(|b| error_symbol_page_body(b, symbol_input))
        .build()
}
