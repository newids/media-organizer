use dioxus::prelude::*;
use dioxus_free_icons::{Icon, icons::{fa_solid_icons, fa_regular_icons, fa_brands_icons}};
use serde::{Serialize, Deserialize};

// Icon Pack Types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IconPack {
    VSCode,
    Material,
    Seti,
    Atom,
    Minimal,
}

impl IconPack {
    pub fn name(&self) -> &'static str {
        match self {
            IconPack::VSCode => "VS Code",
            IconPack::Material => "Material Design",
            IconPack::Seti => "Seti UI",
            IconPack::Atom => "Atom",
            IconPack::Minimal => "Minimal",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            IconPack::VSCode => "Classic VS Code file icons with familiar colors",
            IconPack::Material => "Material Design icons with bold shapes and bright colors",
            IconPack::Seti => "Seti UI theme icons with diamonds and triangular shapes",
            IconPack::Atom => "Atom editor icons with stars and distinctive shapes",
            IconPack::Minimal => "Minimal icons with simple dots and subtle colors",
        }
    }

    pub fn all() -> Vec<IconPack> {
        vec![
            IconPack::VSCode,
            IconPack::Material,
            IconPack::Seti,
            IconPack::Atom,
            IconPack::Minimal,
        ]
    }
}

// Component for displaying file icons
#[component]
pub fn FileIconComponent(file_name: String, extension: Option<String>, is_directory: bool, is_expanded: bool, pack: Option<IconPack>) -> Element {
    let icon_pack = pack.unwrap_or(IconPack::VSCode);

    // Render icons directly based on pack and file type
    match icon_pack {
        IconPack::VSCode => {
            if is_directory {
                if is_expanded {
                    rsx! { Icon { width: 14, height: 14, fill: "#dcb67a", icon: fa_regular_icons::FaFolderOpen } }
                } else {
                    rsx! { Icon { width: 14, height: 14, fill: "#dcb67a", icon: fa_solid_icons::FaFolder } }
                }
            } else {
                // File icons based on extension
                if let Some(ext) = &extension {
                    match ext.to_lowercase().as_str() {
                        "rs" => rsx! { Icon { width: 14, height: 14, fill: "#ce422b", icon: fa_solid_icons::FaCode } },
                        "js" | "mjs" => rsx! { Icon { width: 14, height: 14, fill: "#f7df1e", icon: fa_brands_icons::FaJs } },
                        "ts" => rsx! { Icon { width: 14, height: 14, fill: "#3178c6", icon: fa_solid_icons::FaFileCode } },
                        "py" => rsx! { Icon { width: 14, height: 14, fill: "#3776ab", icon: fa_brands_icons::FaPython } },
                        _ => rsx! { Icon { width: 14, height: 14, fill: "#c5c5c5", icon: fa_regular_icons::FaFile } }
                    }
                } else {
                    rsx! { Icon { width: 14, height: 14, fill: "#c5c5c5", icon: fa_regular_icons::FaFile } }
                }
            }
        },
        IconPack::Material => {
            if is_directory {
                rsx! { Icon { width: 14, height: 14, fill: "#2196F3", icon: fa_solid_icons::FaFolderOpen } }
            } else {
                if let Some(ext) = &extension {
                    match ext.to_lowercase().as_str() {
                        "rs" | "js" | "ts" | "py" => rsx! { Icon { width: 14, height: 14, fill: "#FF9800", icon: fa_solid_icons::FaSquare } },
                        _ => rsx! { Icon { width: 14, height: 14, fill: "#616161", icon: fa_solid_icons::FaCircle } }
                    }
                } else {
                    rsx! { Icon { width: 14, height: 14, fill: "#616161", icon: fa_solid_icons::FaCircle } }
                }
            }
        },
        IconPack::Seti => {
            if is_directory {
                rsx! { Icon { width: 14, height: 14, fill: "#8dc149", icon: fa_solid_icons::FaPlay } }
            } else {
                if let Some(ext) = &extension {
                    match ext.to_lowercase().as_str() {
                        "rs" => rsx! { Icon { width: 14, height: 14, fill: "#8dc149", icon: fa_solid_icons::FaDiamond } },
                        "js" => rsx! { Icon { width: 14, height: 14, fill: "#cbcb41", icon: fa_solid_icons::FaDiamond } },
                        "py" => rsx! { Icon { width: 14, height: 14, fill: "#3572a5", icon: fa_solid_icons::FaDiamond } },
                        _ => rsx! { Icon { width: 14, height: 14, fill: "#41535b", icon: fa_solid_icons::FaDiamond } }
                    }
                } else {
                    rsx! { Icon { width: 14, height: 14, fill: "#41535b", icon: fa_solid_icons::FaDiamond } }
                }
            }
        },
        IconPack::Atom => {
            if is_directory {
                rsx! { Icon { width: 14, height: 14, fill: "#e06c75", icon: fa_solid_icons::FaHeart } }
            } else {
                if let Some(ext) = &extension {
                    match ext.to_lowercase().as_str() {
                        "rs" => rsx! { Icon { width: 14, height: 14, fill: "#e06c75", icon: fa_solid_icons::FaStar } },
                        "js" => rsx! { Icon { width: 14, height: 14, fill: "#d19a66", icon: fa_solid_icons::FaStar } },
                        "py" => rsx! { Icon { width: 14, height: 14, fill: "#98c379", icon: fa_solid_icons::FaStar } },
                        _ => rsx! { Icon { width: 14, height: 14, fill: "#abb2bf", icon: fa_solid_icons::FaStar } }
                    }
                } else {
                    rsx! { Icon { width: 14, height: 14, fill: "#abb2bf", icon: fa_solid_icons::FaStar } }
                }
            }
        },
        IconPack::Minimal => {
            if is_directory {
                rsx! { Icon { width: 14, height: 14, fill: "#aaaaaa", icon: fa_regular_icons::FaCircleDot } }
            } else {
                rsx! { Icon { width: 14, height: 14, fill: "#888888", icon: fa_solid_icons::FaCircleDot } }
            }
        }
    }
}