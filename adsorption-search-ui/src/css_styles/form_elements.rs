use css_style::{
    prelude::*,
    unit::{em, percent, px},
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
