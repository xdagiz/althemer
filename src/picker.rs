use nucleo_picker::{Picker, PickerOptions, render::StrRenderer};

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

    let items = themes
        .iter()
        .map(|t| {
            let marker = if current_path == Some(&t.path) {
                " ●"
            } else {
                ""
            };
            format!("{} {}{}", t.category.icon(), t.name, marker)
        })
        .collect::<Vec<_>>();

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
            Ok(themes.iter().find(|&t| t.name == name).cloned())
        }
        None => Ok(None),
    }
}
