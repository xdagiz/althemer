use nucleo_picker::{Picker, PickerOptions, render::StrRenderer};
use std::collections::HashMap;

use crate::config::AlthemerConfig;
use crate::error::{AlthemerError, Result};
use crate::themes::{Theme, ThemeCategory};

/// Pick a theme from a list
pub fn pick_theme(
    themes: &[Theme],
    current: Option<&Theme>,
    config: &AlthemerConfig,
) -> Result<Option<Theme>> {
    let current_path = current.map(|t| &t.path);

    let mut items = Vec::with_capacity(themes.len());
    let mut name_to_index = HashMap::with_capacity(themes.len());

    for (i, t) in themes.iter().enumerate() {
        let marker = if current_path == Some(&t.path) {
            " ●"
        } else {
            ""
        };
        let display = format!("{} {}{}", t.category.icon(), t.name, marker);
        name_to_index.insert(t.name.as_str(), i);
        items.push(display);
    }

    let mut picker: Picker<String, _> = PickerOptions::new()
        .reversed(config.picker_reversed)
        .highlight(true)
        .sort_results(config.picker_sort_results)
        .picker(StrRenderer);

    picker.extend(items);

    let selection = picker
        .pick()
        .map_err(|e| AlthemerError::InteractiveError(e.to_string()))?;

    match selection {
        Some(sel) => {
            let name = sel
                .trim_end_matches(" ●")
                .trim_start_matches(ThemeCategory::Dark.icon())
                .trim_start_matches(ThemeCategory::Light.icon())
                .trim();
            Ok(name_to_index.get(name).map(|&i| themes[i].clone()))
        }
        None => Ok(None),
    }
}
