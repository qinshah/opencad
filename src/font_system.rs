use bevy::prelude::*;
use bevy_egui::{EguiContexts, egui};
use std::sync::Arc;

pub fn set_heiti(mut contexts: EguiContexts) {
    // Start with the default fonts (we will be adding to them rather than replacing them).
    let mut fonts = egui::FontDefinitions::default();

    // Install my own font (maybe supporting non-latin characters).
    // .ttf and .otf files supported.
    fonts.font_data.insert(
        "my_font".to_owned(),
        Arc::new(egui::FontData::from_static(include_bytes!(
            "../assets/fonts/STHeiti Medium.ttc"
        ))),
    );

    // Put my font first (highest priority) for proportional text:
    fonts
        .families
        .entry(egui::FontFamily::Proportional)
        .or_default()
        .insert(0, "my_font".to_owned());

    // Put my font as last fallback for monospace:
    fonts
        .families
        .entry(egui::FontFamily::Monospace)
        .or_default()
        .push("my_font".to_owned());

    let ctx = contexts.ctx_mut().unwrap();
    ctx.set_fonts(fonts);
    println!("黑体设置成功");
}
