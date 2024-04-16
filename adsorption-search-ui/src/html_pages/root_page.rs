use super::head;
use html::{
    forms::builders::FormBuilder,
    root::{builders::BodyBuilder, Html},
};

pub fn build_root_page() -> Html {
    Html::builder().head(head).body(body).build()
}

fn body(x: &mut BodyBuilder) -> &mut BodyBuilder {
    x.header(|h| h.heading_1(|h1| h1.text("Config of the Program")))
        .form(parameter_form)
        .script(|script| script.src("/assets/my_wasm.js").type_("module"))
}

fn parameter_form(form_builder: &mut FormBuilder) -> &mut FormBuilder {
    let mut x = form_builder
        .id("param_form")
        .action("/submit_form")
        .method("post")
        .enctype("multipart/form-data")
        .paragraph(|para| {
            para.label(|l| {
                l.for_("Upload model file")
                    .text("Upload model file. (Supports: .cell, .msi, .cif)")
            })
        })
        .input(|i| i.type_("file").name("file").required("true"))
        .paragraph(|p| p.label(|l| l.text("Element Symbol")))
        .input(|i| {
            i.type_("text")
                .name("element_symbol")
                .id("element_symbol")
                .required("true")
        })
        .span(|span| span.class("error").id("error-hint"))
        .paragraph(|p| p.label(|l| l.text("Target bondlength (Å)")))
        .input(|i| {
            i.type_("number")
                .name("target_bondlength")
                .required("true")
                .value("0.0")
                .min("0.0")
                .max("10.0")
                .step("0.001")
        });
    for axis in ["x", "y", "z"] {
        x = range_input(x, axis);
    }
    x.input(|input| input.type_("submit").id("submit").value("Submit and Run"))
}

fn range_input<'a>(form_builder: &'a mut FormBuilder, axis: &'a str) -> &'a mut FormBuilder {
    form_builder
        .paragraph(|p| {
            p.label(|l| {
                l.for_(format!("{}-range", axis)).text(format!(
                    "Enter range of {}-axis to search (in fractional coordinate)",
                    axis
                ))
            })
        })
        .input(|i| {
            i.type_("number")
                .name(format!("{}_min", axis))
                .value("0.000")
                .min("0.000")
                .max("1.000")
                .step("0.01")
        })
        .text("-")
        .input(|i| {
            i.type_("number")
                .name(format!("{}_max", axis))
                .value("1.000")
                .min("0.000")
                .max("1.000")
                .step("0.01")
        })
}
