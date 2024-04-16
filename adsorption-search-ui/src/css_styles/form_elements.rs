use css_style::{
    color::{self, palette::FromColor},
    prelude::*,
    unit::{em, percent, px},
    Color,
};

use super::ElementStyle;

pub fn label_style() -> ElementStyle {
    ElementStyle::builder()
        .with_name("label")
        .with_style(
            style()
                .insert("display", "block")
                .and_font(|conf| conf.size(em(1.5)).bold())
                .insert("margin-block-start", "0.67em"),
        )
        .build()
        .unwrap()
}

pub fn input_text_style() -> ElementStyle {
    ElementStyle::builder()
        .with_name("input[type=text], select, textarea")
        .with_style(
            style()
                .and_size(|conf| conf.width(percent(1.0)))
                .and_padding(|conf| conf.all(px(12)))
                .insert("cursor", "pointer"),
        )
        .build()
        .unwrap()
}

pub fn input_number_style() -> ElementStyle {
    ElementStyle::builder()
        .with_name("input[type=number]")
        .with_style(
            style()
                .and_size(|size| size.max_width(percent(1.0)).min_width(0.45))
                .and_padding(|pad| pad.all(px(12))),
        )
        .build()
        .unwrap()
}

pub fn all_box_sizing() -> ElementStyle {
    ElementStyle::builder()
        .with_name("*")
        .with_style(style().insert("box-sizing", "border-box"))
        .build()
        .unwrap()
}

pub fn error_message() -> ElementStyle {
    ElementStyle::builder()
        .with_name(".error")
        .with_style(
            style()
                .and_size(|size| size.width(percent(1.0)))
                .and_padding(|pad| pad.zero())
                .and_font(|font| font.size(percent(0.8)))
                .insert("background-color", "#900")
                .insert("color", "white")
                .insert("border-radius", "0 0 5px 5px"),
        )
        .build()
        .unwrap()
}

pub fn error_active() -> ElementStyle {
    ElementStyle::builder()
        .with_name(".error.active")
        .with_style(style().and_padding(|pad| pad.all(em(0.3))))
        .build()
        .unwrap()
}
