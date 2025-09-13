use dioxus::prelude::*;
use dioxus_free_icons::{Icon, icons::fa_solid_icons, icons::fa_regular_icons, icons::fa_brands_icons};
use std::collections::HashMap;
use once_cell::sync::Lazy;

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

pub struct FileIcon {
    pub icon: fn() -> Element,
    pub color: &'static str,
}

pub static ICON_REGISTRY: Lazy<HashMap<IconType, FileIcon>> = Lazy::new(|| {
    let mut icons = HashMap::new();
    
    // Default icons
    icons.insert(IconType::File, FileIcon {
        icon: || rsx! { Icon { width: 14, height: 14, fill: "currentColor", icon: fa_regular_icons::FaFile } },
        color: "#c5c5c5",
    });
    
    icons.insert(IconType::Folder, FileIcon {
        icon: || rsx! { Icon { width: 14, height: 14, fill: "currentColor", icon: fa_solid_icons::FaFolder } },
        color: "#dcb67a",
    });
    
    icons.insert(IconType::FolderOpen, FileIcon {
        icon: || rsx! { Icon { width: 14, height: 14, fill: "currentColor", icon: fa_regular_icons::FaFolderOpen } },
        color: "#dcb67a",
    });
    
    // Programming Languages
    icons.insert(IconType::Rust, FileIcon {
        icon: || rsx! { Icon { width: 14, height: 14, fill: "currentColor", icon: fa_solid_icons::FaCode } },
        color: "#ce422b",
    });
    
    icons.insert(IconType::JavaScript, FileIcon {
        icon: || rsx! { Icon { width: 14, height: 14, fill: "currentColor", icon: fa_brands_icons::FaJs } },
        color: "#f7df1e",
    });
    
    icons.insert(IconType::TypeScript, FileIcon {
        icon: || rsx! { Icon { width: 14, height: 14, fill: "currentColor", icon: fa_solid_icons::FaFileCode } },
        color: "#3178c6",
    });
    
    icons.insert(IconType::Python, FileIcon {
        icon: || rsx! { Icon { width: 14, height: 14, fill: "currentColor", icon: fa_brands_icons::FaPython } },
        color: "#3776ab",
    });
    
    icons.insert(IconType::Go, FileIcon {
        icon: || rsx! { Icon { width: 14, height: 14, fill: "currentColor", icon: fa_solid_icons::FaCode } },
        color: "#00add8",
    });
    
    icons.insert(IconType::Java, FileIcon {
        icon: || rsx! { Icon { width: 14, height: 14, fill: "currentColor", icon: fa_brands_icons::FaJava } },
        color: "#007396",
    });
    
    icons.insert(IconType::CSharp, FileIcon {
        icon: || rsx! { Icon { width: 14, height: 14, fill: "currentColor", icon: fa_solid_icons::FaFileCode } },
        color: "#239120",
    });
    
    icons.insert(IconType::Cpp, FileIcon {
        icon: || rsx! { Icon { width: 14, height: 14, fill: "currentColor", icon: fa_solid_icons::FaFileCode } },
        color: "#00599c",
    });
    
    icons.insert(IconType::C, FileIcon {
        icon: || rsx! { Icon { width: 14, height: 14, fill: "currentColor", icon: fa_solid_icons::FaFileCode } },
        color: "#555555",
    });
    
    icons.insert(IconType::Swift, FileIcon {
        icon: || rsx! { Icon { width: 14, height: 14, fill: "currentColor", icon: fa_brands_icons::FaSwift } },
        color: "#fa7343",
    });
    
    icons.insert(IconType::Kotlin, FileIcon {
        icon: || rsx! { Icon { width: 14, height: 14, fill: "currentColor", icon: fa_solid_icons::FaFileCode } },
        color: "#7f52ff",
    });
    
    icons.insert(IconType::Ruby, FileIcon {
        icon: || rsx! { Icon { width: 14, height: 14, fill: "currentColor", icon: fa_solid_icons::FaGem } },
        color: "#cc342d",
    });
    
    icons.insert(IconType::Php, FileIcon {
        icon: || rsx! { Icon { width: 14, height: 14, fill: "currentColor", icon: fa_brands_icons::FaPhp } },
        color: "#777bb4",
    });
    
    // Web Technologies
    icons.insert(IconType::Html, FileIcon {
        icon: || rsx! { Icon { width: 14, height: 14, fill: "currentColor", icon: fa_brands_icons::FaHtml5 } },
        color: "#e34c26",
    });
    
    icons.insert(IconType::Css, FileIcon {
        icon: || rsx! { Icon { width: 14, height: 14, fill: "currentColor", icon: fa_brands_icons::FaCss3Alt } },
        color: "#1572b6",
    });
    
    icons.insert(IconType::Sass, FileIcon {
        icon: || rsx! { Icon { width: 14, height: 14, fill: "currentColor", icon: fa_brands_icons::FaSass } },
        color: "#cc6699",
    });
    
    icons.insert(IconType::Vue, FileIcon {
        icon: || rsx! { Icon { width: 14, height: 14, fill: "currentColor", icon: fa_brands_icons::FaVuejs } },
        color: "#4fc08d",
    });
    
    icons.insert(IconType::React, FileIcon {
        icon: || rsx! { Icon { width: 14, height: 14, fill: "currentColor", icon: fa_brands_icons::FaReact } },
        color: "#61dafb",
    });
    
    icons.insert(IconType::Angular, FileIcon {
        icon: || rsx! { Icon { width: 14, height: 14, fill: "currentColor", icon: fa_brands_icons::FaAngular } },
        color: "#dd0031",
    });
    
    icons.insert(IconType::Svelte, FileIcon {
        icon: || rsx! { Icon { width: 14, height: 14, fill: "currentColor", icon: fa_solid_icons::FaFileCode } },
        color: "#ff3e00",
    });
    
    // Data & Config
    icons.insert(IconType::Json, FileIcon {
        icon: || rsx! { Icon { width: 14, height: 14, fill: "currentColor", icon: fa_solid_icons::FaFileLines } },
        color: "#cbcb41",
    });
    
    icons.insert(IconType::Xml, FileIcon {
        icon: || rsx! { Icon { width: 14, height: 14, fill: "currentColor", icon: fa_solid_icons::FaCode } },
        color: "#ff6600",
    });
    
    icons.insert(IconType::Yaml, FileIcon {
        icon: || rsx! { Icon { width: 14, height: 14, fill: "currentColor", icon: fa_solid_icons::FaFileLines } },
        color: "#cb171e",
    });
    
    icons.insert(IconType::Toml, FileIcon {
        icon: || rsx! { Icon { width: 14, height: 14, fill: "currentColor", icon: fa_solid_icons::FaFileLines } },
        color: "#9c4121",
    });
    
    icons.insert(IconType::Ini, FileIcon {
        icon: || rsx! { Icon { width: 14, height: 14, fill: "currentColor", icon: fa_solid_icons::FaGear } },
        color: "#6d8086",
    });
    
    icons.insert(IconType::Env, FileIcon {
        icon: || rsx! { Icon { width: 14, height: 14, fill: "currentColor", icon: fa_solid_icons::FaKey } },
        color: "#edd94c",
    });
    
    // Documents
    icons.insert(IconType::Markdown, FileIcon {
        icon: || rsx! { Icon { width: 14, height: 14, fill: "currentColor", icon: fa_brands_icons::FaMarkdown } },
        color: "#519aba",
    });
    
    icons.insert(IconType::Text, FileIcon {
        icon: || rsx! { Icon { width: 14, height: 14, fill: "currentColor", icon: fa_regular_icons::FaFileLines } },
        color: "#c5c5c5",
    });
    
    icons.insert(IconType::Pdf, FileIcon {
        icon: || rsx! { Icon { width: 14, height: 14, fill: "currentColor", icon: fa_regular_icons::FaFilePdf } },
        color: "#d9524f",
    });
    
    icons.insert(IconType::Word, FileIcon {
        icon: || rsx! { Icon { width: 14, height: 14, fill: "currentColor", icon: fa_regular_icons::FaFileWord } },
        color: "#2b579a",
    });
    
    icons.insert(IconType::Excel, FileIcon {
        icon: || rsx! { Icon { width: 14, height: 14, fill: "currentColor", icon: fa_regular_icons::FaFileExcel } },
        color: "#217346",
    });
    
    icons.insert(IconType::PowerPoint, FileIcon {
        icon: || rsx! { Icon { width: 14, height: 14, fill: "currentColor", icon: fa_regular_icons::FaFilePowerpoint } },
        color: "#d04423",
    });
    
    // Media
    icons.insert(IconType::Image, FileIcon {
        icon: || rsx! { Icon { width: 14, height: 14, fill: "currentColor", icon: fa_regular_icons::FaFileImage } },
        color: "#a074c4",
    });
    
    icons.insert(IconType::Video, FileIcon {
        icon: || rsx! { Icon { width: 14, height: 14, fill: "currentColor", icon: fa_regular_icons::FaFileVideo } },
        color: "#db5860",
    });
    
    icons.insert(IconType::Audio, FileIcon {
        icon: || rsx! { Icon { width: 14, height: 14, fill: "currentColor", icon: fa_regular_icons::FaFileAudio } },
        color: "#5aa7ff",
    });
    
    // Archives
    icons.insert(IconType::Archive, FileIcon {
        icon: || rsx! { Icon { width: 14, height: 14, fill: "currentColor", icon: fa_regular_icons::FaFileZipper } },
        color: "#b8a038",
    });
    
    // Build & Package
    icons.insert(IconType::Package, FileIcon {
        icon: || rsx! { Icon { width: 14, height: 14, fill: "currentColor", icon: fa_solid_icons::FaBox } },
        color: "#cb3837",
    });
    
    icons.insert(IconType::Lock, FileIcon {
        icon: || rsx! { Icon { width: 14, height: 14, fill: "currentColor", icon: fa_solid_icons::FaLock } },
        color: "#4e4e4e",
    });
    
    icons.insert(IconType::Docker, FileIcon {
        icon: || rsx! { Icon { width: 14, height: 14, fill: "currentColor", icon: fa_brands_icons::FaDocker } },
        color: "#0db7ed",
    });
    
    icons.insert(IconType::Git, FileIcon {
        icon: || rsx! { Icon { width: 14, height: 14, fill: "currentColor", icon: fa_brands_icons::FaGitAlt } },
        color: "#f05032",
    });
    
    // Shell & Scripts
    icons.insert(IconType::Shell, FileIcon {
        icon: || rsx! { Icon { width: 14, height: 14, fill: "currentColor", icon: fa_solid_icons::FaTerminal } },
        color: "#4eaa25",
    });
    
    icons.insert(IconType::PowerShell, FileIcon {
        icon: || rsx! { Icon { width: 14, height: 14, fill: "currentColor", icon: fa_solid_icons::FaTerminal } },
        color: "#012456",
    });
    
    icons.insert(IconType::Batch, FileIcon {
        icon: || rsx! { Icon { width: 14, height: 14, fill: "currentColor", icon: fa_solid_icons::FaTerminal } },
        color: "#c1f12e",
    });
    
    // Database
    icons.insert(IconType::Database, FileIcon {
        icon: || rsx! { Icon { width: 14, height: 14, fill: "currentColor", icon: fa_solid_icons::FaDatabase } },
        color: "#db7533",
    });
    
    icons.insert(IconType::Sql, FileIcon {
        icon: || rsx! { Icon { width: 14, height: 14, fill: "currentColor", icon: fa_solid_icons::FaDatabase } },
        color: "#ffca28",
    });
    
    // Other
    icons.insert(IconType::Binary, FileIcon {
        icon: || rsx! { Icon { width: 14, height: 14, fill: "currentColor", icon: fa_solid_icons::FaMicrochip } },
        color: "#9e9e9e",
    });
    
    icons.insert(IconType::Certificate, FileIcon {
        icon: || rsx! { Icon { width: 14, height: 14, fill: "currentColor", icon: fa_solid_icons::FaCertificate } },
        color: "#ff9800",
    });
    
    icons.insert(IconType::Key, FileIcon {
        icon: || rsx! { Icon { width: 14, height: 14, fill: "currentColor", icon: fa_solid_icons::FaKey } },
        color: "#ffc107",
    });
    
    icons.insert(IconType::License, FileIcon {
        icon: || rsx! { Icon { width: 14, height: 14, fill: "currentColor", icon: fa_solid_icons::FaScaleBalanced } },
        color: "#cc0000",
    });
    
    icons.insert(IconType::Readme, FileIcon {
        icon: || rsx! { Icon { width: 14, height: 14, fill: "currentColor", icon: fa_solid_icons::FaBookOpen } },
        color: "#42a5f5",
    });
    
    icons.insert(IconType::Ignore, FileIcon {
        icon: || rsx! { Icon { width: 14, height: 14, fill: "currentColor", icon: fa_solid_icons::FaBan } },
        color: "#757575",
    });
    
    icons.insert(IconType::EditorConfig, FileIcon {
        icon: || rsx! { Icon { width: 14, height: 14, fill: "currentColor", icon: fa_solid_icons::FaGear } },
        color: "#fafafa",
    });
    
    icons
});

pub static FILE_EXTENSION_MAP: Lazy<HashMap<&'static str, IconType>> = Lazy::new(|| {
    let mut map = HashMap::new();
    
    // Programming Languages
    map.insert("rs", IconType::Rust);
    map.insert("js", IconType::JavaScript);
    map.insert("mjs", IconType::JavaScript);
    map.insert("cjs", IconType::JavaScript);
    map.insert("jsx", IconType::React);
    map.insert("ts", IconType::TypeScript);
    map.insert("tsx", IconType::React);
    map.insert("py", IconType::Python);
    map.insert("pyc", IconType::Python);
    map.insert("pyo", IconType::Python);
    map.insert("pyw", IconType::Python);
    map.insert("go", IconType::Go);
    map.insert("java", IconType::Java);
    map.insert("class", IconType::Java);
    map.insert("jar", IconType::Java);
    map.insert("cs", IconType::CSharp);
    map.insert("cpp", IconType::Cpp);
    map.insert("cxx", IconType::Cpp);
    map.insert("cc", IconType::Cpp);
    map.insert("c++", IconType::Cpp);
    map.insert("c", IconType::C);
    map.insert("h", IconType::C);
    map.insert("hpp", IconType::Cpp);
    map.insert("hxx", IconType::Cpp);
    map.insert("swift", IconType::Swift);
    map.insert("kt", IconType::Kotlin);
    map.insert("kts", IconType::Kotlin);
    map.insert("rb", IconType::Ruby);
    map.insert("php", IconType::Php);
    
    // Web Technologies
    map.insert("html", IconType::Html);
    map.insert("htm", IconType::Html);
    map.insert("xhtml", IconType::Html);
    map.insert("css", IconType::Css);
    map.insert("scss", IconType::Sass);
    map.insert("sass", IconType::Sass);
    map.insert("less", IconType::Css);
    map.insert("vue", IconType::Vue);
    map.insert("svelte", IconType::Svelte);
    
    // Data & Config
    map.insert("json", IconType::Json);
    map.insert("jsonc", IconType::Json);
    map.insert("json5", IconType::Json);
    map.insert("xml", IconType::Xml);
    map.insert("yaml", IconType::Yaml);
    map.insert("yml", IconType::Yaml);
    map.insert("toml", IconType::Toml);
    map.insert("ini", IconType::Ini);
    map.insert("cfg", IconType::Ini);
    map.insert("conf", IconType::Ini);
    map.insert("config", IconType::Ini);
    map.insert("env", IconType::Env);
    
    // Documents
    map.insert("md", IconType::Markdown);
    map.insert("markdown", IconType::Markdown);
    map.insert("txt", IconType::Text);
    map.insert("text", IconType::Text);
    map.insert("pdf", IconType::Pdf);
    map.insert("doc", IconType::Word);
    map.insert("docx", IconType::Word);
    map.insert("xls", IconType::Excel);
    map.insert("xlsx", IconType::Excel);
    map.insert("ppt", IconType::PowerPoint);
    map.insert("pptx", IconType::PowerPoint);
    
    // Media
    map.insert("jpg", IconType::Image);
    map.insert("jpeg", IconType::Image);
    map.insert("png", IconType::Image);
    map.insert("gif", IconType::Image);
    map.insert("bmp", IconType::Image);
    map.insert("svg", IconType::Image);
    map.insert("webp", IconType::Image);
    map.insert("ico", IconType::Image);
    map.insert("mp4", IconType::Video);
    map.insert("avi", IconType::Video);
    map.insert("mov", IconType::Video);
    map.insert("wmv", IconType::Video);
    map.insert("flv", IconType::Video);
    map.insert("webm", IconType::Video);
    map.insert("mkv", IconType::Video);
    map.insert("mp3", IconType::Audio);
    map.insert("wav", IconType::Audio);
    map.insert("flac", IconType::Audio);
    map.insert("aac", IconType::Audio);
    map.insert("ogg", IconType::Audio);
    map.insert("wma", IconType::Audio);
    
    // Archives
    map.insert("zip", IconType::Archive);
    map.insert("tar", IconType::Archive);
    map.insert("gz", IconType::Archive);
    map.insert("bz2", IconType::Archive);
    map.insert("xz", IconType::Archive);
    map.insert("rar", IconType::Archive);
    map.insert("7z", IconType::Archive);
    
    // Shell & Scripts
    map.insert("sh", IconType::Shell);
    map.insert("bash", IconType::Shell);
    map.insert("zsh", IconType::Shell);
    map.insert("fish", IconType::Shell);
    map.insert("ps1", IconType::PowerShell);
    map.insert("psm1", IconType::PowerShell);
    map.insert("psd1", IconType::PowerShell);
    map.insert("bat", IconType::Batch);
    map.insert("cmd", IconType::Batch);
    
    // Database
    map.insert("db", IconType::Database);
    map.insert("sqlite", IconType::Database);
    map.insert("sqlite3", IconType::Database);
    map.insert("sql", IconType::Sql);
    
    // Other
    map.insert("exe", IconType::Binary);
    map.insert("dll", IconType::Binary);
    map.insert("so", IconType::Binary);
    map.insert("dylib", IconType::Binary);
    map.insert("crt", IconType::Certificate);
    map.insert("cer", IconType::Certificate);
    map.insert("pem", IconType::Certificate);
    map.insert("key", IconType::Key);
    map.insert("pub", IconType::Key);
    
    map
});

pub static FILE_NAME_MAP: Lazy<HashMap<&'static str, IconType>> = Lazy::new(|| {
    let mut map = HashMap::new();
    
    // Special file names
    map.insert("package.json", IconType::Package);
    map.insert("package-lock.json", IconType::Lock);
    map.insert("yarn.lock", IconType::Lock);
    map.insert("pnpm-lock.yaml", IconType::Lock);
    map.insert("cargo.toml", IconType::Package);
    map.insert("cargo.lock", IconType::Lock);
    map.insert("dockerfile", IconType::Docker);
    map.insert("docker-compose.yml", IconType::Docker);
    map.insert("docker-compose.yaml", IconType::Docker);
    map.insert(".gitignore", IconType::Ignore);
    map.insert(".gitattributes", IconType::Git);
    map.insert(".gitmodules", IconType::Git);
    map.insert("readme.md", IconType::Readme);
    map.insert("readme.txt", IconType::Readme);
    map.insert("readme", IconType::Readme);
    map.insert("license", IconType::License);
    map.insert("license.md", IconType::License);
    map.insert("license.txt", IconType::License);
    map.insert(".editorconfig", IconType::EditorConfig);
    map.insert(".env", IconType::Env);
    map.insert(".env.local", IconType::Env);
    map.insert(".env.development", IconType::Env);
    map.insert(".env.production", IconType::Env);
    
    map
});

pub fn get_icon_for_file(file_name: &str, extension: Option<&str>, is_directory: bool, is_expanded: bool) -> (IconType, &'static str) {
    if is_directory {
        let icon_type = if is_expanded { IconType::FolderOpen } else { IconType::Folder };
        let icon = ICON_REGISTRY.get(&icon_type).unwrap();
        return (icon_type, icon.color);
    }
    
    // Check special file names first
    let lower_name = file_name.to_lowercase();
    if let Some(&icon_type) = FILE_NAME_MAP.get(lower_name.as_str()) {
        let icon = ICON_REGISTRY.get(&icon_type).unwrap();
        return (icon_type, icon.color);
    }
    
    // Check by extension
    if let Some(ext) = extension {
        let lower_ext = ext.to_lowercase();
        if let Some(&icon_type) = FILE_EXTENSION_MAP.get(lower_ext.as_str()) {
            let icon = ICON_REGISTRY.get(&icon_type).unwrap();
            return (icon_type, icon.color);
        }
    }
    
    // Default file icon
    let icon = ICON_REGISTRY.get(&IconType::File).unwrap();
    (IconType::File, icon.color)
}

#[component]
pub fn FileIconComponent(file_name: String, extension: Option<String>, is_directory: bool, is_expanded: bool) -> Element {
    let (icon_type, color) = get_icon_for_file(&file_name, extension.as_deref(), is_directory, is_expanded);
    let icon = ICON_REGISTRY.get(&icon_type).unwrap();
    
    rsx! {
        div {
            style: "color: {color}; display: inline-flex; align-items: center;",
            {(icon.icon)()}
        }
    }
}