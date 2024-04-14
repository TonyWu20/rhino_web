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
