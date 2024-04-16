use castep_periodic_table::{data::ELEMENT_TABLE, element::LookupElement};
use gloo::{
    dialogs::alert,
    events::{EventListener, EventListenerOptions},
    utils::document,
};
use wasm_bindgen::prelude::*;
use web_sys::{wasm_bindgen::JsCast, Element};

#[wasm_bindgen(start)]
pub fn main() {
    listen_text_input_input();
    listen_text_input_paste();
    listen_text_input_change();
    listen_form_submit_symbol_input();
    init_panic_hook();
}

fn listen_text_input_change() {
    let input_field = document()
        .query_selector(r#"input[name="element_symbol"]"#)
        .expect("No 'element_symbol' field")
        .unwrap()
        .dyn_into::<web_sys::HtmlInputElement>()
        .unwrap();
    let on_change = EventListener::new(&input_field, "change", move |event| {
        let element_symbol_input = event
            .target()
            .unwrap()
            .dyn_into::<web_sys::HtmlInputElement>()
            .unwrap()
            .value();
        instant_error(&element_symbol_input);
    });
    on_change.forget();
}

fn listen_text_input_paste() {
    let input_field = document()
        .query_selector(r#"input[name="element_symbol"]"#)
        .expect("No 'element_symbol' field")
        .unwrap()
        .dyn_into::<web_sys::HtmlInputElement>()
        .unwrap();

    let on_paste = EventListener::new(&input_field, "change", move |event| {
        let element_symbol_input = event
            .target()
            .unwrap()
            .dyn_into::<web_sys::HtmlInputElement>()
            .unwrap()
            .value();
        instant_error(&element_symbol_input);
    });
    on_paste.forget()
}

fn listen_text_input_input() {
    let input_field = document()
        .query_selector(r#"input[name="element_symbol"]"#)
        .expect("No 'element_symbol' field")
        .unwrap()
        .dyn_into::<web_sys::HtmlInputElement>()
        .unwrap();

    let on_input = EventListener::new(&input_field, "input", move |event| {
        let element_symbol_input = event
            .target()
            .unwrap()
            .dyn_into::<web_sys::HtmlInputElement>()
            .unwrap()
            .value();
        instant_error(&element_symbol_input);
    });
    on_input.forget()
}

#[wasm_bindgen]
pub fn listen_form_submit_symbol_input() {
    let form = document()
        .query_selector("#param_form")
        .expect("No form in the document")
        .unwrap()
        .dyn_into::<web_sys::HtmlFormElement>()
        .unwrap();
    let enable_prevent_default = EventListenerOptions::enable_prevent_default();
    let on_submit =
        EventListener::new_with_options(&form, "submit", enable_prevent_default, move |event| {
            let element_symbol_input = document()
                .query_selector(r#"input[name="element_symbol"]"#)
                .expect("No 'element_symbol' field")
                .unwrap()
                .dyn_into::<web_sys::HtmlInputElement>()
                .unwrap()
                .value();
            if !check_symbol_input(&element_symbol_input) {
                event.prevent_default(); // Stop form submitting to server
                event.stop_propagation();
                alert_error(&element_symbol_input);
            } else {
                let error_span = document()
                    .get_element_by_id("error-hint")
                    .expect("No `error-hint` span");
                error_span.set_text_content(None);
                error_span.set_class_name("error");
            }
        });
    on_submit.forget()
}

fn instant_error(element_symbol: &str) {
    let error_span = document()
        .get_element_by_id("error-hint")
        .expect("No `error-hint` span");
    if !check_symbol_input(element_symbol) {
        show_error(&error_span);
    } else {
        error_span.set_text_content(None);
        error_span.set_class_name("error");
    }
}

fn alert_error(element_symbol: &str) {
    let error_span = document()
        .get_element_by_id("error-hint")
        .expect("No `error-hint` span");
    if !check_symbol_input(element_symbol) {
        let message = format!("Invalid element symbol! Input: {}", element_symbol);
        alert(&message);
        error_span.set_text_content(Some("Invalid element symbol!"));
        error_span.set_class_name("error active");
    } else {
        error_span.set_text_content(None);
        error_span.set_class_name("error");
    }
}

fn show_error(error_element: &Element) {
    error_element.set_text_content(Some("Invalid element symbol!"));
    error_element.set_class_name("error active");
}

#[wasm_bindgen]
pub fn check_symbol_input(input: &str) -> bool {
    ELEMENT_TABLE.get_by_symbol(input).is_some()
}

#[wasm_bindgen]
pub fn init_panic_hook() {
    console_error_panic_hook::set_once();
}
