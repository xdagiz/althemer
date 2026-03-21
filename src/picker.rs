use nucleo_picker::{Picker, PickerOptions, render::StrRenderer};

use crate::themes::{Theme, ThemeCategory};

/// Pick a theme from a list
pub fn pick_theme(themes: &[Theme], current: Option<&Theme>) -> Option<Theme> {
    let mut sorted_themes = themes.to_vec();

    // sort uppercase first, then alphabetically (case-insensitive)
    sorted_themes.sort_by(|a, b| {
        let a_has_upper = a.name.chars().any(|c| c.is_uppercase());
        let b_has_upper = b.name.chars().any(|c| c.is_uppercase());

        match (a_has_upper, b_has_upper) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
        }
    });

    let items: Vec<String> = sorted_themes
        .iter()
        .map(|t| {
            if current.is_some_and(|c| c.name == t.name) {
                format!("{} {} ●", t.category.icon(), t.name)
            } else {
                format!("{} {}", t.category.icon(), t.name)
            }
        })
        .collect();

    // TODO: add config option for `reversed` and `sort_results`
    let mut picker: Picker<String, _> = PickerOptions::new()
        .reversed(true)
        .highlight(true)
        // .sort_results(false)
        .picker(StrRenderer);

    picker.extend(items);

    if let Ok(Some(selection)) = picker.pick() {
        let name = selection
            .trim_end_matches(" ●")
            .trim_start_matches(ThemeCategory::Dark.icon())
            .trim_start_matches(ThemeCategory::Light.icon())
            .trim();
        sorted_themes.into_iter().find(|t| t.name == name)
    } else {
        println!("No theme selected");
        None
    }
}
