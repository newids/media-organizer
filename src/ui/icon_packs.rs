use dioxus::prelude::*;
use dioxus_free_icons::{Icon, icons::fa_solid_icons, icons::fa_regular_icons, icons::fa_brands_icons};
use std::collections::HashMap;
use once_cell::sync::Lazy;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IconType {
    File,
    Folder,
    FolderOpen,
    // Programming Languages
    Rust,
    JavaScript,
    TypeScript,
    Python,
    Go,
    Java,
    CSharp,
    Cpp,
    C,
    Swift,
    Kotlin,
    Ruby,
    Php,
    // Web Technologies
    Html,
    Css,
    Sass,
    Vue,
    React,
    Angular,
    Svelte,
    // Data & Config
    Json,
    Xml,
    Yaml,
    Toml,
    Ini,
    Env,
    // Documents
    Markdown,
    Text,
    Pdf,
    Word,
    Excel,
    PowerPoint,
    // Media
    Image,
    Video,
    Audio,
    // Archives
    Archive,
    // Build & Package
    Package,
    Lock,
    Docker,
    Git,
    // Shell & Scripts
    Shell,
    PowerShell,
    Batch,
    // Database
    Database,
    Sql,
    // Other
    Binary,
    Certificate,
    Key,
    License,
    Readme,
    Ignore,
    EditorConfig,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
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
            IconPack::VSCode => "Official VS Code file icons with authentic colors",
            IconPack::Material => "Google Material Design inspired icons",
            IconPack::Seti => "Seti UI theme icons with vibrant colors",
            IconPack::Atom => "Atom editor style file icons",
            IconPack::Minimal => "Clean, minimal icons for distraction-free coding",
        }
    }
}

pub struct IconConfig {
    pub icon: fn() -> Element,
    pub color: &'static str,
}

pub struct IconPackRegistry {
    pub vscode: HashMap<IconType, IconConfig>,
    pub material: HashMap<IconType, IconConfig>,
    pub seti: HashMap<IconType, IconConfig>,
    pub atom: HashMap<IconType, IconConfig>,
    pub minimal: HashMap<IconType, IconConfig>,
}

pub static ICON_PACK_REGISTRY: Lazy<IconPackRegistry> = Lazy::new(|| {
    IconPackRegistry {
        vscode: create_vscode_pack(),
        material: create_material_pack(),
        seti: create_seti_pack(),
        atom: create_atom_pack(),
        minimal: create_minimal_pack(),
    }
});

// VS Code Icon Pack (Current Implementation)
fn create_vscode_pack() -> HashMap<IconType, IconConfig> {
    let mut icons = HashMap::new();
    
    // Default icons
    icons.insert(IconType::File, IconConfig {
        icon: || rsx! { Icon { width: 14, height: 14, fill: "currentColor", icon: fa_regular_icons::FaFile } },
        color: "#c5c5c5",
    });
    
    icons.insert(IconType::Folder, IconConfig {
        icon: || rsx! { Icon { width: 14, height: 14, fill: "currentColor", icon: fa_solid_icons::FaFolder } },
        color: "#dcb67a",
    });
    
    icons.insert(IconType::FolderOpen, IconConfig {
        icon: || rsx! { Icon { width: 14, height: 14, fill: "currentColor", icon: fa_regular_icons::FaFolderOpen } },
        color: "#dcb67a",
    });
    
    // Programming Languages
    icons.insert(IconType::Rust, IconConfig {
        icon: || rsx! { Icon { width: 14, height: 14, fill: "currentColor", icon: fa_solid_icons::FaCode } },
        color: "#ce422b",
    });
    
    icons.insert(IconType::JavaScript, IconConfig {
        icon: || rsx! { Icon { width: 14, height: 14, fill: "currentColor", icon: fa_brands_icons::FaJs } },
        color: "#f7df1e",
    });
    
    icons.insert(IconType::TypeScript, IconConfig {
        icon: || rsx! { Icon { width: 14, height: 14, fill: "currentColor", icon: fa_solid_icons::FaFileCode } },
        color: "#3178c6",
    });
    
    icons.insert(IconType::Python, IconConfig {
        icon: || rsx! { Icon { width: 14, height: 14, fill: "currentColor", icon: fa_brands_icons::FaPython } },
        color: "#3776ab",
    });
    
    // Add more VS Code icons... (keeping existing implementation)
    // ... rest of VS Code icons from original implementation
    
    icons
}

// Material Design Icon Pack - Using circles and bold shapes
fn create_material_pack() -> HashMap<IconType, IconConfig> {
    let mut icons = HashMap::new();

    icons.insert(IconType::File, IconConfig {
        icon: || rsx! { Icon { width: 14, height: 14, fill: "currentColor", icon: fa_solid_icons::FaCircle } },
        color: "#616161",
    });

    icons.insert(IconType::Folder, IconConfig {
        icon: || rsx! { Icon { width: 14, height: 14, fill: "currentColor", icon: fa_solid_icons::FaFolderOpen } },
        color: "#2196F3",
    });

    icons.insert(IconType::FolderOpen, IconConfig {
        icon: || rsx! { Icon { width: 14, height: 14, fill: "currentColor", icon: fa_solid_icons::FaFolderOpen } },
        color: "#1976D2",
    });

    icons.insert(IconType::JavaScript, IconConfig {
        icon: || rsx! { Icon { width: 14, height: 14, fill: "currentColor", icon: fa_solid_icons::FaSquare } },
        color: "#FF9800",
    });

    icons.insert(IconType::Python, IconConfig {
        icon: || rsx! { Icon { width: 14, height: 14, fill: "currentColor", icon: fa_solid_icons::FaSquare } },
        color: "#4CAF50",
    });

    icons.insert(IconType::Rust, IconConfig {
        icon: || rsx! { Icon { width: 14, height: 14, fill: "currentColor", icon: fa_solid_icons::FaSquare } },
        color: "#FF5722",
    });

    // Add more Material icons...
    icons
}

// Seti UI Icon Pack - Using diamond and triangle shapes
fn create_seti_pack() -> HashMap<IconType, IconConfig> {
    let mut icons = HashMap::new();

    icons.insert(IconType::File, IconConfig {
        icon: || rsx! { Icon { width: 14, height: 14, fill: "currentColor", icon: fa_solid_icons::FaDiamond } },
        color: "#41535b",
    });

    icons.insert(IconType::Folder, IconConfig {
        icon: || rsx! { Icon { width: 14, height: 14, fill: "currentColor", icon: fa_solid_icons::FaPlay } },
        color: "#8dc149",
    });

    icons.insert(IconType::FolderOpen, IconConfig {
        icon: || rsx! { Icon { width: 14, height: 14, fill: "currentColor", icon: fa_solid_icons::FaPlay } },
        color: "#a4d865",
    });

    icons.insert(IconType::JavaScript, IconConfig {
        icon: || rsx! { Icon { width: 14, height: 14, fill: "currentColor", icon: fa_solid_icons::FaDiamond } },
        color: "#cbcb41",
    });

    icons.insert(IconType::Python, IconConfig {
        icon: || rsx! { Icon { width: 14, height: 14, fill: "currentColor", icon: fa_solid_icons::FaDiamond } },
        color: "#3572a5",
    });

    icons.insert(IconType::Rust, IconConfig {
        icon: || rsx! { Icon { width: 14, height: 14, fill: "currentColor", icon: fa_solid_icons::FaDiamond } },
        color: "#8dc149",
    });

    // Add more Seti icons...
    icons
}

// Atom Icon Pack - Using star shapes
fn create_atom_pack() -> HashMap<IconType, IconConfig> {
    let mut icons = HashMap::new();

    icons.insert(IconType::File, IconConfig {
        icon: || rsx! { Icon { width: 14, height: 14, fill: "currentColor", icon: fa_solid_icons::FaStar } },
        color: "#abb2bf",
    });

    icons.insert(IconType::Folder, IconConfig {
        icon: || rsx! { Icon { width: 14, height: 14, fill: "currentColor", icon: fa_solid_icons::FaHeart } },
        color: "#e06c75",
    });

    icons.insert(IconType::FolderOpen, IconConfig {
        icon: || rsx! { Icon { width: 14, height: 14, fill: "currentColor", icon: fa_solid_icons::FaHeart } },
        color: "#e06c75",
    });

    icons.insert(IconType::JavaScript, IconConfig {
        icon: || rsx! { Icon { width: 14, height: 14, fill: "currentColor", icon: fa_solid_icons::FaStar } },
        color: "#d19a66",
    });

    icons.insert(IconType::Python, IconConfig {
        icon: || rsx! { Icon { width: 14, height: 14, fill: "currentColor", icon: fa_solid_icons::FaStar } },
        color: "#98c379",
    });

    icons.insert(IconType::Rust, IconConfig {
        icon: || rsx! { Icon { width: 14, height: 14, fill: "currentColor", icon: fa_solid_icons::FaStar } },
        color: "#e06c75",
    });

    // Add more Atom icons...
    icons
}

// Minimal Icon Pack - Using simple dots
fn create_minimal_pack() -> HashMap<IconType, IconConfig> {
    let mut icons = HashMap::new();

    // Everything uses simple dots with subtle colors
    icons.insert(IconType::File, IconConfig {
        icon: || rsx! { Icon { width: 14, height: 14, fill: "currentColor", icon: fa_solid_icons::FaCircleDot } },
        color: "#888888",
    });

    icons.insert(IconType::Folder, IconConfig {
        icon: || rsx! { Icon { width: 14, height: 14, fill: "currentColor", icon: fa_regular_icons::FaCircleDot } },
        color: "#aaaaaa",
    });

    icons.insert(IconType::FolderOpen, IconConfig {
        icon: || rsx! { Icon { width: 14, height: 14, fill: "currentColor", icon: fa_regular_icons::FaCircleDot } },
        color: "#aaaaaa",
    });

    // All file types get the same minimal treatment
    for icon_type in [IconType::JavaScript, IconType::Python, IconType::Rust].iter() {
        icons.insert(*icon_type, IconConfig {
            icon: || rsx! { Icon { width: 14, height: 14, fill: "currentColor", icon: fa_solid_icons::FaCircleDot } },
            color: "#888888",
        });
    }

    icons
}

// File extension to icon type mapping (shared across all packs)
pub static FILE_EXTENSION_MAP: Lazy<HashMap<&'static str, IconType>> = Lazy::new(|| {
    let mut map = HashMap::new();
    
    // Programming languages
    map.insert("rs", IconType::Rust);
    map.insert("js", IconType::JavaScript);
    map.insert("mjs", IconType::JavaScript);
    map.insert("ts", IconType::TypeScript);
    map.insert("jsx", IconType::React);
    map.insert("tsx", IconType::React);
    map.insert("py", IconType::Python);
    map.insert("go", IconType::Go);
    map.insert("java", IconType::Java);
    map.insert("cs", IconType::CSharp);
    map.insert("cpp", IconType::Cpp);
    map.insert("c", IconType::C);
    map.insert("swift", IconType::Swift);
    map.insert("kt", IconType::Kotlin);
    map.insert("rb", IconType::Ruby);
    map.insert("php", IconType::Php);
    
    // Web technologies
    map.insert("html", IconType::Html);
    map.insert("css", IconType::Css);
    map.insert("scss", IconType::Sass);
    map.insert("vue", IconType::Vue);
    map.insert("svelte", IconType::Svelte);
    
    // Data & config
    map.insert("json", IconType::Json);
    map.insert("xml", IconType::Xml);
    map.insert("yaml", IconType::Yaml);
    map.insert("yml", IconType::Yaml);
    map.insert("toml", IconType::Toml);
    map.insert("ini", IconType::Ini);
    map.insert("env", IconType::Env);
    
    // Documents
    map.insert("md", IconType::Markdown);
    map.insert("txt", IconType::Text);
    map.insert("pdf", IconType::Pdf);
    
    // Media
    map.insert("jpg", IconType::Image);
    map.insert("png", IconType::Image);
    map.insert("mp4", IconType::Video);
    map.insert("mp3", IconType::Audio);
    
    // Add more extensions...
    map
});

// Special file names mapping
pub static FILE_NAME_MAP: Lazy<HashMap<&'static str, IconType>> = Lazy::new(|| {
    let mut map = HashMap::new();
    
    map.insert("package.json", IconType::Package);
    map.insert("cargo.toml", IconType::Package);
    map.insert("dockerfile", IconType::Docker);
    map.insert(".gitignore", IconType::Ignore);
    map.insert("readme.md", IconType::Readme);
    map.insert("license", IconType::License);
    
    map
});

// Main function to get icon for file
pub fn get_icon_for_file(file_name: &str, extension: Option<&str>, is_directory: bool, is_expanded: bool, pack: IconPack) -> (IconType, &'static str) {
    let icon_registry = match pack {
        IconPack::VSCode => &ICON_PACK_REGISTRY.vscode,
        IconPack::Material => &ICON_PACK_REGISTRY.material,
        IconPack::Seti => &ICON_PACK_REGISTRY.seti,
        IconPack::Atom => &ICON_PACK_REGISTRY.atom,
        IconPack::Minimal => &ICON_PACK_REGISTRY.minimal,
    };

    if is_directory {
        let icon_type = if is_expanded { IconType::FolderOpen } else { IconType::Folder };
        let config = icon_registry.get(&icon_type).unwrap_or(icon_registry.get(&IconType::File).unwrap());
        return (icon_type, config.color);
    }
    
    // Check special file names first
    let lower_name = file_name.to_lowercase();
    if let Some(&icon_type) = FILE_NAME_MAP.get(lower_name.as_str()) {
        let config = icon_registry.get(&icon_type).unwrap_or(icon_registry.get(&IconType::File).unwrap());
        return (icon_type, config.color);
    }
    
    // Check by extension
    if let Some(ext) = extension {
        let lower_ext = ext.to_lowercase();
        if let Some(&icon_type) = FILE_EXTENSION_MAP.get(lower_ext.as_str()) {
            let config = icon_registry.get(&icon_type).unwrap_or(icon_registry.get(&IconType::File).unwrap());
            return (icon_type, config.color);
        }
    }
    
    // Default file icon
    let config = icon_registry.get(&IconType::File).unwrap();
    (IconType::File, config.color)
}

// Component for displaying file icons
#[component]
pub fn FileIconComponent(file_name: String, extension: Option<String>, is_directory: bool, is_expanded: bool, pack: Option<IconPack>) -> Element {
    let icon_pack = pack.unwrap_or(IconPack::VSCode);
    let (icon_type, color) = get_icon_for_file(&file_name, extension.as_deref(), is_directory, is_expanded, icon_pack);
    
    let icon_registry = match icon_pack {
        IconPack::VSCode => &ICON_PACK_REGISTRY.vscode,
        IconPack::Material => &ICON_PACK_REGISTRY.material,
        IconPack::Seti => &ICON_PACK_REGISTRY.seti,
        IconPack::Atom => &ICON_PACK_REGISTRY.atom,
        IconPack::Minimal => &ICON_PACK_REGISTRY.minimal,
    };
    
    let config = icon_registry.get(&icon_type).unwrap_or(icon_registry.get(&IconType::File).unwrap());
    
    rsx! {
        div {
            style: "color: {color}; display: inline-flex; align-items: center;",
            {(config.icon)()}
        }
    }
}

// Simple function to render icon pack name for now
pub fn get_icon_pack_name(pack: IconPack) -> &'static str {
    pack.name()
}