use css_style::{font::Family, prelude::*, unit::percent};

use super::ElementStyle;

pub fn body_style() -> ElementStyle {
    ElementStyle::builder()
        .with_name("body")
        .with_style(
            style()
                .and_font(|font| {
                    font.family(Family::from("Helvetica Neue, sans-serif"))
                        .size(percent(1.0))
                })
                .and_margin(|conf| conf.left(percent(0.25)).right(percent(0.25))),
        )
        .build()
        .unwrap()
}
