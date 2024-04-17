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

fn form_error_page_body(
    body: &mut BodyBuilder,
    error_title: String,
    error_content: String,
) -> &mut BodyBuilder {
    body.header(|h| h.heading_1(|h1| h1.text(error_title)))
        .paragraph(|p| p.text(error_content))
        .anchor(|anchor| anchor.href("/").button(|b| b.text("Back")))
}

pub fn build_error_symbol_page(symbol_input: String) -> Html {
    Html::builder()
        .head(head)
        .body(|b| error_symbol_page_body(b, symbol_input))
        .build()
}

pub fn build_error_format_page() -> Html {
    Html::builder()
        .head(head)
        .body(|b| {
            b.header(|h| h.heading_1(|h1| h1.text("Error: Uploaded unsupported file format.")))
                .paragraph(|p| p.text("Current supported formats:"))
                .unordered_list(|ul| {
                    ul.list_item(|item| item.text(".cif"))
                        .list_item(|item| item.text(".msi (Accelrys Cerius II)"))
                        .list_item(|item| item.text(".cell (castep)"))
                })
                .paragraph(|p| p.text("You might have submitted a corrupted file or the submitted content is not a real file (e.g. a soft link or a shortcut to a file). Please double check the file to be uploaded."))
                .anchor(|anchor| anchor.href("/").button(|b| b.text("Back")))
        })
        .build()
}

pub fn build_form_error_general_page(error_title: String, error_content: String) -> Html {
    Html::builder()
        .head(head)
        .body(|b| form_error_page_body(b, error_title, error_content))
        .build()
}
