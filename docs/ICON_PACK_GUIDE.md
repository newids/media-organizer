# Icon Pack Management Guide

MediaOrganizer supports multiple icon packs to customize the appearance of file icons in the file tree. This guide explains how to change icons, create new icon packs, and manage the icon system.

## 🎨 Available Icon Packs

### 1. VS Code (Default)
- **Location**: `src/ui/icon_packs.rs` → `create_vscode_pack()`
- **Colors**: Authentic VS Code colors
- **Style**: Official VS Code file icons with FontAwesome fallbacks

### 2. Material Design
- **Location**: `src/ui/icon_packs.rs` → `create_material_pack()`
- **Colors**: Google Material Design color palette
- **Style**: Clean, modern Google-inspired icons

### 3. Seti UI
- **Location**: `src/ui/icon_packs.rs` → `create_seti_pack()`
- **Colors**: Vibrant, colorful theme
- **Style**: Popular atom/vscode theme adaptation

### 4. Atom
- **Location**: `src/ui/icon_packs.rs` → `create_atom_pack()`
- **Colors**: One Dark Pro inspired
- **Style**: GitHub Atom editor style

### 5. Minimal
- **Location**: `src/ui/icon_packs.rs` → `create_minimal_pack()`
- **Colors**: Subtle grays
- **Style**: Distraction-free, uniform appearance

## 🔧 How to Change Icons

### Method 1: Using the Settings Panel (Recommended)

1. **Open Settings**: Menu → Preferences or `Cmd/Ctrl + ,`
2. **Navigate to Icon Settings**: Look for "File Icon Theme" section
3. **Select Pack**: Click on your preferred icon pack
4. **Preview**: See live preview of icons at the bottom
5. **Apply**: Changes are saved automatically

### Method 2: Programmatically

```rust
use crate::ui::{IconManager, IconPack};

// Get icon manager
let mut icon_manager = use_icon_manager();

// Change to Material Design pack
icon_manager.change_pack(IconPack::Material);
```

## 📝 Creating Custom Icon Packs

### Step 1: Add New Icon Pack Enum

In `src/ui/icon_packs.rs`, add your pack to the `IconPack` enum:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum IconPack {
    VSCode,
    Material,
    Seti,
    Atom,
    Minimal,
    MyCustomPack,  // Add your pack here
}
```

### Step 2: Implement Pack Methods

```rust
impl IconPack {
    pub fn name(&self) -> &'static str {
        match self {
            // ... existing cases
            IconPack::MyCustomPack => "My Custom Pack",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            // ... existing cases
            IconPack::MyCustomPack => "My awesome custom icon pack",
        }
    }
}
```

### Step 3: Create Icon Pack Function

```rust
fn create_my_custom_pack() -> HashMap<IconType, IconConfig> {
    let mut icons = HashMap::new();
    
    // Default file icon
    icons.insert(IconType::File, IconConfig {
        icon: || rsx! { Icon { 
            width: 14, 
            height: 14, 
            fill: "currentColor", 
            icon: fa_regular_icons::FaFile 
        }},
        color: "#your_color_here",
    });
    
    // Folder icons
    icons.insert(IconType::Folder, IconConfig {
        icon: || rsx! { Icon { 
            width: 14, 
            height: 14, 
            fill: "currentColor", 
            icon: fa_solid_icons::FaFolder 
        }},
        color: "#folder_color",
    });
    
    // Add specific file type icons
    icons.insert(IconType::JavaScript, IconConfig {
        icon: || rsx! { Icon { 
            width: 14, 
            height: 14, 
            fill: "currentColor", 
            icon: fa_brands_icons::FaJs 
        }},
        color: "#js_color",
    });
    
    // Add more icons for each IconType you want to support
    
    icons
}
```

### Step 4: Register Pack in Registry

```rust
pub static ICON_PACK_REGISTRY: Lazy<IconPackRegistry> = Lazy::new(|| {
    IconPackRegistry {
        vscode: create_vscode_pack(),
        material: create_material_pack(),
        seti: create_seti_pack(),
        atom: create_atom_pack(),
        minimal: create_minimal_pack(),
        my_custom_pack: create_my_custom_pack(), // Add your pack
    }
});
```

### Step 5: Update Registry Struct

```rust
pub struct IconPackRegistry {
    pub vscode: HashMap<IconType, IconConfig>,
    pub material: HashMap<IconType, IconConfig>,
    pub seti: HashMap<IconType, IconConfig>,
    pub atom: HashMap<IconType, IconConfig>,
    pub minimal: HashMap<IconType, IconConfig>,
    pub my_custom_pack: HashMap<IconType, IconConfig>, // Add field
}
```

### Step 6: Update Pack Matching

```rust
pub fn get_icon_for_file(..., pack: IconPack) -> (IconType, &'static str) {
    let icon_registry = match pack {
        IconPack::VSCode => &ICON_PACK_REGISTRY.vscode,
        IconPack::Material => &ICON_PACK_REGISTRY.material,
        IconPack::Seti => &ICON_PACK_REGISTRY.seti,
        IconPack::Atom => &ICON_PACK_REGISTRY.atom,
        IconPack::Minimal => &ICON_PACK_REGISTRY.minimal,
        IconPack::MyCustomPack => &ICON_PACK_REGISTRY.my_custom_pack, // Add case
    };
    
    // ... rest of function
}
```

## 🎯 Adding New File Types

### Step 1: Add IconType Variant

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IconType {
    // ... existing types
    MyNewFileType,  // Add your file type
}
```

### Step 2: Map Extensions

```rust
pub static FILE_EXTENSION_MAP: Lazy<HashMap<&'static str, IconType>> = Lazy::new(|| {
    let mut map = HashMap::new();
    
    // ... existing mappings
    map.insert("mynewext", IconType::MyNewFileType);
    
    map
});
```

### Step 3: Add to All Icon Packs

Update each icon pack creation function to include your new file type:

```rust
fn create_vscode_pack() -> HashMap<IconType, IconConfig> {
    let mut icons = HashMap::new();
    
    // ... existing icons
    icons.insert(IconType::MyNewFileType, IconConfig {
        icon: || rsx! { /* your icon */ },
        color: "#color",
    });
    
    icons
}
```

## 📁 File Structure

```
src/ui/
├── icons.rs              # Original icon system (deprecated)
├── icon_packs.rs          # Main icon pack definitions
├── icon_manager.rs        # Settings and management
└── components/
    └── file_tree.rs       # Uses icon system
```

## 🛠️ Advanced Customization

### Custom Icon Sources

You can use different icon sources beyond FontAwesome:

```rust
// SVG icons
icons.insert(IconType::Rust, IconConfig {
    icon: || rsx! {
        svg {
            width: "14",
            height: "14",
            viewBox: "0 0 24 24",
            path {
                d: "M12 2L2 7v10l10 5 10-5V7L12 2z",
                fill: "currentColor"
            }
        }
    },
    color: "#ce422b",
});

// Unicode symbols
icons.insert(IconType::Text, IconConfig {
    icon: || rsx! {
        span {
            style: "font-size: 14px; font-weight: bold;",
            "📄"
        }
    },
    color: "#888888",
});
```

### Dynamic Colors

Colors can be computed dynamically:

```rust
fn get_dynamic_color(file_name: &str) -> &'static str {
    if file_name.starts_with("test_") {
        "#ff6b6b"  // Red for test files
    } else if file_name.contains("config") {
        "#4ecdc4"  // Teal for config files
    } else {
        "#95a5a6"  // Default gray
    }
}
```

## 🔍 Troubleshooting

### Icons Not Showing

1. **Check Pack Registration**: Ensure your pack is added to `ICON_PACK_REGISTRY`
2. **Verify Icon Component**: Make sure `FileIconComponent` is using the correct pack
3. **FontAwesome Dependencies**: Ensure required FontAwesome features are enabled in `Cargo.toml`

### Colors Not Applying

1. **CSS Variables**: Check if you're using CSS custom properties correctly
2. **Color Format**: Ensure colors are in valid CSS format (`#rrggbb`)
3. **Theme Override**: Check if theme is overriding icon colors

### Performance Issues

1. **Lazy Loading**: Icon registries use `Lazy` for performance
2. **Icon Caching**: Consider caching icon components for frequently used files
3. **Pack Switching**: Pack changes require component re-renders

## 📊 Settings Persistence

Icon settings are automatically saved to:
- **macOS**: `~/Library/Application Support/MediaOrganizer/icon_settings.json`
- **Windows**: `%APPDATA%\MediaOrganizer\icon_settings.json`
- **Linux**: `~/.config/MediaOrganizer/icon_settings.json`

Settings include:
- Current icon pack
- Icon size preference
- Show/hide file extensions
- Custom icon paths (future feature)

## 🚀 Future Enhancements

Planned features for icon management:

1. **Custom Icon Importing**: Load icons from external files/URLs
2. **Icon Pack Marketplace**: Download community-created packs
3. **Per-Project Icons**: Different icon packs per project/workspace
4. **Icon Animations**: Subtle animations for file operations
5. **Accessibility Options**: High contrast and colorblind-friendly modes

## 📚 API Reference

### Key Functions

- `get_icon_for_file(name, ext, is_dir, expanded, pack)` - Get icon type and color
- `IconManager::change_pack(pack)` - Switch icon pack
- `IconSettings::load()` - Load persisted settings
- `use_icon_manager()` - Hook to access icon manager

### Components

- `FileIconComponent` - Renders file icons
- `IconSettingsPanel` - Settings UI
- `IconPackSelector` - Pack selection UI
- `IconManagerProvider` - Context provider

This guide provides everything needed to customize and extend the icon system in MediaOrganizer!