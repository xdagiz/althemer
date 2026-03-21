use nucleo_picker::{Picker, PickerOptions, render::StrRenderer};

use crate::config::AlthemerConfig;
use crate::themes::{Theme, ThemeCategory};

/// Pick a theme from a list
pub fn pick_theme(
    themes: &[Theme],
    current: Option<&Theme>,
    config: &AlthemerConfig,
) -> Option<Theme> {
    let current_name = current.map(|t| t.name.as_str());

    let items: Vec<String> = themes
        .iter()
        .map(|t| {
            let marker = if current_name == Some(t.name.as_str()) {
                " ●"
            } else {
                ""
            };
            format!("{} {}{}", t.category.icon(), t.name, marker)
        })
        .collect();

    let mut picker: Picker<String, _> = PickerOptions::new()
        .reversed(config.picker_reversed)
        .highlight(true)
        .sort_results(config.picker_sort_results)
        .picker(StrRenderer);

    picker.extend(items);

    picker.pick().ok().flatten().and_then(|selection| {
        let name = selection
            .trim_end_matches(" ●")
            .trim_start_matches(ThemeCategory::Dark.icon())
            .trim_start_matches(ThemeCategory::Light.icon())
            .trim();
        themes.iter().find(|t| t.name == name).cloned()
    })
}
