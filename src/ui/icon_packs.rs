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
                // VS Code-style file icons based on extension
                if let Some(ext) = &extension {
                    match ext.to_lowercase().as_str() {
                        // Programming Languages
                        "rs" => rsx! { Icon { width: 14, height: 14, fill: "#ce422b", icon: fa_solid_icons::FaCode } },
                        "js" | "mjs" => rsx! { Icon { width: 14, height: 14, fill: "#f7df1e", icon: fa_brands_icons::FaJs } },
                        "ts" | "tsx" => rsx! { Icon { width: 14, height: 14, fill: "#3178c6", icon: fa_solid_icons::FaFileCode } },
                        "jsx" => rsx! { Icon { width: 14, height: 14, fill: "#61dafb", icon: fa_brands_icons::FaReact } },
                        "py" => rsx! { Icon { width: 14, height: 14, fill: "#3776ab", icon: fa_brands_icons::FaPython } },
                        "java" => rsx! { Icon { width: 14, height: 14, fill: "#ed8b00", icon: fa_brands_icons::FaJava } },
                        "cpp" | "cc" | "cxx" => rsx! { Icon { width: 14, height: 14, fill: "#00599c", icon: fa_solid_icons::FaCode } },
                        "c" => rsx! { Icon { width: 14, height: 14, fill: "#a8b9cc", icon: fa_solid_icons::FaCode } },
                        "cs" => rsx! { Icon { width: 14, height: 14, fill: "#239120", icon: fa_solid_icons::FaCode } },
                        "php" => rsx! { Icon { width: 14, height: 14, fill: "#777bb4", icon: fa_brands_icons::FaPhp } },
                        "rb" => rsx! { Icon { width: 14, height: 14, fill: "#cc342d", icon: fa_solid_icons::FaGem } },
                        "go" => rsx! { Icon { width: 14, height: 14, fill: "#00add8", icon: fa_solid_icons::FaCode } },
                        "swift" => rsx! { Icon { width: 14, height: 14, fill: "#fa7343", icon: fa_brands_icons::FaSwift } },
                        "kt" | "kts" => rsx! { Icon { width: 14, height: 14, fill: "#7f52ff", icon: fa_solid_icons::FaCode } },
                        "dart" => rsx! { Icon { width: 14, height: 14, fill: "#0175c2", icon: fa_solid_icons::FaCode } },
                        "scala" => rsx! { Icon { width: 14, height: 14, fill: "#dc322f", icon: fa_solid_icons::FaCode } },
                        "sh" | "bash" | "zsh" => rsx! { Icon { width: 14, height: 14, fill: "#89e051", icon: fa_solid_icons::FaTerminal } },

                        // Web Technologies
                        "html" | "htm" => rsx! { Icon { width: 14, height: 14, fill: "#e34c26", icon: fa_brands_icons::FaHtml5 } },
                        "css" => rsx! { Icon { width: 14, height: 14, fill: "#1572b6", icon: fa_brands_icons::FaCss3Alt } },
                        "scss" | "sass" => rsx! { Icon { width: 14, height: 14, fill: "#cf649a", icon: fa_brands_icons::FaSass } },
                        "less" => rsx! { Icon { width: 14, height: 14, fill: "#1d365d", icon: fa_solid_icons::FaPalette } },
                        "vue" => rsx! { Icon { width: 14, height: 14, fill: "#4fc08d", icon: fa_brands_icons::FaVuejs } },
                        "svelte" => rsx! { Icon { width: 14, height: 14, fill: "#ff3e00", icon: fa_solid_icons::FaCode } },

                        // Config & Data
                        "json" => rsx! { Icon { width: 14, height: 14, fill: "#ffd700", icon: fa_solid_icons::FaFileCode } },
                        "xml" => rsx! { Icon { width: 14, height: 14, fill: "#ff6600", icon: fa_solid_icons::FaCode } },
                        "yaml" | "yml" => rsx! { Icon { width: 14, height: 14, fill: "#cb171e", icon: fa_solid_icons::FaList } },
                        "toml" => rsx! { Icon { width: 14, height: 14, fill: "#9c4221", icon: fa_solid_icons::FaGear } },
                        "ini" | "cfg" => rsx! { Icon { width: 14, height: 14, fill: "#6d8086", icon: fa_solid_icons::FaGear } },
                        "env" => rsx! { Icon { width: 14, height: 14, fill: "#ecd53f", icon: fa_solid_icons::FaKey } },

                        // Documentation
                        "md" => rsx! { Icon { width: 14, height: 14, fill: "#083fa1", icon: fa_brands_icons::FaMarkdown } },
                        "txt" => rsx! { Icon { width: 14, height: 14, fill: "#c5c5c5", icon: fa_regular_icons::FaFileLines } },
                        "doc" | "docx" => rsx! { Icon { width: 14, height: 14, fill: "#2b579a", icon: fa_solid_icons::FaFileWord } },
                        "pdf" => rsx! { Icon { width: 14, height: 14, fill: "#ff0000", icon: fa_regular_icons::FaFilePdf } },

                        // Images
                        "png" | "jpg" | "jpeg" | "gif" | "bmp" | "webp" => rsx! { Icon { width: 14, height: 14, fill: "#4caf50", icon: fa_regular_icons::FaImage } },
                        "svg" => rsx! { Icon { width: 14, height: 14, fill: "#ffb13b", icon: fa_solid_icons::FaVectorSquare } },
                        "ico" => rsx! { Icon { width: 14, height: 14, fill: "#4caf50", icon: fa_regular_icons::FaImage } },

                        // Audio & Video
                        "mp3" | "wav" | "flac" | "ogg" | "m4a" => rsx! { Icon { width: 14, height: 14, fill: "#ff9800", icon: fa_solid_icons::FaVolumeHigh } },
                        "mp4" | "avi" | "mkv" | "mov" | "webm" => rsx! { Icon { width: 14, height: 14, fill: "#e91e63", icon: fa_solid_icons::FaVideo } },

                        // Archives
                        "zip" | "rar" | "7z" | "tar" | "gz" | "bz2" => rsx! { Icon { width: 14, height: 14, fill: "#795548", icon: fa_regular_icons::FaFileZipper } },

                        // Package managers & Build
                        "lock" => rsx! { Icon { width: 14, height: 14, fill: "#f44336", icon: fa_solid_icons::FaLock } },
                        "npmrc" => rsx! { Icon { width: 14, height: 14, fill: "#cb3837", icon: fa_brands_icons::FaNpm } },
                        "gitignore" => rsx! { Icon { width: 14, height: 14, fill: "#f05032", icon: fa_brands_icons::FaGit } },
                        "dockerfile" => rsx! { Icon { width: 14, height: 14, fill: "#2496ed", icon: fa_brands_icons::FaDocker } },

                        // Default file icon
                        _ => rsx! { Icon { width: 14, height: 14, fill: "#c5c5c5", icon: fa_regular_icons::FaFile } }
                    }
                } else {
                    // Handle special files without extensions
                    match file_name.to_lowercase().as_str() {
                        "cargo.toml" | "cargo.lock" => rsx! { Icon { width: 14, height: 14, fill: "#ce422b", icon: fa_solid_icons::FaCode } },
                        "package.json" | "package-lock.json" => rsx! { Icon { width: 14, height: 14, fill: "#cb3837", icon: fa_brands_icons::FaNpm } },
                        "tsconfig.json" => rsx! { Icon { width: 14, height: 14, fill: "#3178c6", icon: fa_solid_icons::FaFileCode } },
                        "webpack.config.js" => rsx! { Icon { width: 14, height: 14, fill: "#8dd6f9", icon: fa_solid_icons::FaCube } },
                        "dockerfile" => rsx! { Icon { width: 14, height: 14, fill: "#2496ed", icon: fa_brands_icons::FaDocker } },
                        "makefile" => rsx! { Icon { width: 14, height: 14, fill: "#427819", icon: fa_solid_icons::FaGear } },
                        "readme" | "readme.md" => rsx! { Icon { width: 14, height: 14, fill: "#083fa1", icon: fa_solid_icons::FaInfo } },
                        "license" | "licence" => rsx! { Icon { width: 14, height: 14, fill: "#d4af37", icon: fa_solid_icons::FaScroll } },
                        ".gitignore" | ".gitattributes" => rsx! { Icon { width: 14, height: 14, fill: "#f05032", icon: fa_brands_icons::FaGit } },
                        ".env" | ".env.local" | ".env.example" => rsx! { Icon { width: 14, height: 14, fill: "#ecd53f", icon: fa_solid_icons::FaKey } },
                        _ => rsx! { Icon { width: 14, height: 14, fill: "#c5c5c5", icon: fa_regular_icons::FaFile } }
                    }
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