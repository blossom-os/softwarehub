use base64::{engine::general_purpose, Engine as _};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
pub struct KdeTheme {
    pub decoration_theme: String,
    pub colors: ThemeColors,
    pub button_icons: ButtonIcons,
    pub titlebar_height: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ThemeColors {
    pub titlebar_bg: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ButtonIcons {
    pub close: Option<String>,
    pub maximize: Option<String>,
    pub minimize: Option<String>,
    pub restore: Option<String>,
}

pub fn get_kde_theme() -> Result<KdeTheme, String> {
    let config_dir = std::env::var("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            let mut home = PathBuf::from(std::env::var("HOME").unwrap_or_default());
            home.push(".config");
            home
        });

    let decoration_theme = get_decoration_theme(&config_dir.join("kwinrc"))?;
    let colors = get_theme_colors(&config_dir.join("kdeglobals"))?;
    let button_icons = get_button_icons(&decoration_theme)?;

    Ok(KdeTheme {
        decoration_theme,
        colors,
        button_icons,
        titlebar_height: 30,
    })
}

fn get_decoration_theme(kwinrc_path: &PathBuf) -> Result<String, String> {
    if !kwinrc_path.exists() {
        return Ok("Breeze".to_string());
    }

    let content =
        fs::read_to_string(kwinrc_path).map_err(|e| format!("Failed to read kwinrc: {}", e))?;
    let mut in_section = false;

    for line in content.lines() {
        if line.starts_with("[org.kde.kdecoration2]") {
            in_section = true;
            continue;
        }
        if line.starts_with('[') {
            in_section = false;
            continue;
        }
        if in_section && line.starts_with("theme=") {
            return Ok(line
                .split('=')
                .nth(1)
                .unwrap_or("Breeze")
                .trim()
                .to_string());
        }
    }

    Ok("Breeze".to_string())
}

fn get_theme_colors(kdeglobals_path: &PathBuf) -> Result<ThemeColors, String> {
    let mut titlebar_bg = "#3daee9".to_string();

    if kdeglobals_path.exists() {
        if let Ok(content) = fs::read_to_string(kdeglobals_path) {
            let mut in_wm = false;
            for line in content.lines() {
                if line.starts_with("[WM]") {
                    in_wm = true;
                    continue;
                }
                if line.starts_with('[') {
                    in_wm = false;
                    continue;
                }
                if in_wm && line.starts_with("activeBackground=") {
                    titlebar_bg = parse_color(line.split('=').nth(1).unwrap_or(""));
                    break;
                }
            }
        }
    }

    Ok(ThemeColors { titlebar_bg })
}

fn parse_color(color: &str) -> String {
    let color = color.trim();
    if color.starts_with('#') {
        return color.to_string();
    }
    let parts: Vec<&str> = color.split(',').collect();
    if parts.len() == 3 {
        if let (Ok(r), Ok(g), Ok(b)) = (
            parts[0].trim().parse::<u8>(),
            parts[1].trim().parse::<u8>(),
            parts[2].trim().parse::<u8>(),
        ) {
            return format!("#{:02x}{:02x}{:02x}", r, g, b);
        }
    }
    "#3daee9".to_string()
}

fn get_button_icons(theme: &str) -> Result<ButtonIcons, String> {
    let theme_dir = PathBuf::from("/usr/share/kwin/decorations").join(theme);

    let buttons = if theme_dir.exists() {
        ButtonIcons {
            close: find_icon(&theme_dir, "close"),
            maximize: find_icon(&theme_dir, "maximize"),
            minimize: find_icon(&theme_dir, "minimize"),
            restore: find_icon(&theme_dir, "restore"),
        }
    } else {
        ButtonIcons {
            close: None,
            maximize: None,
            minimize: None,
            restore: None,
        }
    };

    Ok(ButtonIcons {
        close: buttons.close.or_else(|| find_system_icon("window-close")),
        maximize: buttons
            .maximize
            .or_else(|| find_system_icon("window-maximize")),
        minimize: buttons
            .minimize
            .or_else(|| find_system_icon("window-minimize")),
        restore: buttons
            .restore
            .or_else(|| find_system_icon("window-restore")),
    })
}

fn find_icon(theme_dir: &PathBuf, name: &str) -> Option<String> {
    for size in &["22", "24"] {
        for ext in &["svg", "png"] {
            let path = theme_dir.join(format!("{}{}.{}", name, size, ext));
            if path.exists() {
                return encode_icon(&path);
            }
        }
    }
    None
}

fn find_system_icon(name: &str) -> Option<String> {
    for theme in &["breeze", "breeze-dark", "Adwaita"] {
        for size in &["22", "16", "24"] {
            for ext in &["svg", "png"] {
                let path = PathBuf::from("/usr/share/icons")
                    .join(theme)
                    .join("actions")
                    .join(size)
                    .join(format!("{}.{}", name, ext));
                if path.exists() {
                    return encode_icon(&path);
                }
            }
        }
    }
    None
}

fn encode_icon(path: &PathBuf) -> Option<String> {
    let content = fs::read(path).ok()?;
    let ext = path.extension()?.to_str()?;
    let mime = if ext == "svg" {
        "image/svg+xml"
    } else {
        "image/png"
    };
    let base64 = general_purpose::STANDARD.encode(&content);
    Some(format!("data:{};base64,{}", mime, base64))
}
