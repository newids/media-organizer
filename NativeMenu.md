# Native Menu Structure - MediaOrganizer

This document describes the current native macOS menu bar structure and functionality for the MediaOrganizer application. Edit this file to request menu changes and new functionality.

## Current Menu Structure

### 1. **Media Organizer** (App Menu)
```
Media Organizer
├── About Media Organizer
├── Check for Updates...
├── ────────────────────────────── (separator)
├── Preferences                  [System submenu]
├── ────────────────────────────── (separator)
├── Services                     [System submenu]
├── ────────────────────────────── (separator)
├── Hide Media Organizer         [⌘H]
├── Hide Others                  [⌘⌥H]
├── Show All
├── ────────────────────────────── (separator)
└── Quit Media Organizer         [⌘Q]
```

**Current Status:**
- ✅ **About Media Organizer** - Placeholder (TODO: Implement about dialog)
- ✅ **Check for Updates** - Placeholder (TODO: Implement update check)
- [ ] **Preference** - Open settings page
- ✅ **Services, Hide, Show All, Quit** - System-managed items (working)

---

### 2. **File Menu**
```
File
├── Open Folder...               [⌘O]
├── New Window                   [⌘N]
├── ────────────────────────────── (separator)
├── New Folder                   [⌘⇧N]
├── New File                     [⌘⇧T]
├── ────────────────────────────── (separator)
├── Refresh                      [⌘R]
├── Show Hidden Files            [⌘⇧.]
├── ────────────────────────────── (separator)
├── Open                         [⌘]
├── Open With...                 [⌘⇧]
└── Show in Finder               [⌘⌥R]
```

**Current Status:**
- ✅ **Open Folder...** - **IMPLEMENTED** - Opens native folder picker, updates file tree
- [ ] **New Window** - Open a new window or new instance
- [ ] **New Folder** - Make a new folder in the folder of 'file tree panel'
- [ ] **New File** - Make a new file in the folder of 'file tree panel'
- ✅ **Refresh** - **IMPLEMENTED** - Refreshes current directory view  
- ✅ **Show Hidden Files** - **IMPLEMENTED** - Toggles visibility of hidden files with persistence
- [ ] **Open** - Open a selected file with a system default app in the 'file tree pannel', when olny a file selected
- [ ] **Open With...** - Open a selected file in the 'file tree pannel', when olny a file selected 
- [ ] **Show in Finder** - Open Finder(MacOS) or File Explorer(Windows) with a folder which is placed selected file or current folder

---

### 3. **Edit Menu**
```
Edit
├── Undo                         [⌘Z]
├── Redo                         [⌘⇧Z]
├── ────────────────────────────── (separator)
├── Cut                          [⌘X]
├── Copy                         [⌘C]
├── Paste                        [⌘V]
├── ────────────────────────────── (separator)
├── Select All                   [⌘A]
├── Clear Selection              [⌘⇧A]
├── ────────────────────────────── (separator)
├── Copy to...                   [⌘⇧C]
├── Move to...                   [⌘⇧M]
├── ────────────────────────────── (separator)
├── Delete                       [⌫]
├── Rename                       [↩]
├── Duplicate                    [⌘D]
├── ────────────────────────────── (separator)
└── Settings...                  [⌘,]
```

**Current Status:**
- ✅ **Undo, Redo, Cut, Copy, Paste, Select All** - System-managed items (working)
- ✅ **Clear Selection** - **IMPLEMENTED** - Clears all selected files in the file tree
- ❌ **Copy to...** - Placeholder (TODO: Implement copy to dialog)
- ❌ **Move to...** - Placeholder (TODO: Implement move to dialog)
- ❌ **Delete** - Placeholder (TODO: Implement delete confirmation and action)
- ❌ **Rename** - Placeholder (TODO: Implement inline rename functionality)
- [ ] **Duplicate** - Duplicate file which is selected in 'file tree panell'
- ✅ **Settings...** - **IMPLEMENTED** - Opens settings dialog with theme and font options

---

### 4. **View Menu**
```
View
├── Toggle Sidebar               [⌘B]
├── Toggle Panel                 [⌘J]
├── ────────────────────────────── (separator)
├── Light Theme
├── Dark Theme
└── Auto Theme
```

**Current Status:**
- ❌ **Toggle Sidebar** - Placeholder (TODO: Implement sidebar toggle)
- ❌ **Toggle Panel** - Placeholder (TODO: Implement panel toggle)
- ✅ **Light Theme** - **IMPLEMENTED** - Switches to VSCode Light+ theme with persistence
- ✅ **Dark Theme** - **IMPLEMENTED** - Switches to VSCode Dark+ theme with persistence
- ✅ **Auto Theme** - **IMPLEMENTED** - Auto-detects system preference and applies appropriate theme

---

### 5. **Help Menu**
```
Help
├── Keyboard Shortcuts
└── Media Organizer Help
```

**Current Status:**
- ✅ **Keyboard Shortcuts** - **IMPLEMENTED** - Opens comprehensive keyboard shortcuts cheat sheet
- ❌ **Media Organizer Help** - Placeholder (TODO: Implement help documentation)

---

## Implementation Details

### Working Features

#### **1. Settings Dialog (Edit Menu)**
- **Location:** `src/main.rs:256-261` (menu handler), `src/ui/phase2_app.rs` (dialog integration)
- **Functionality:**
  - Opens comprehensive settings dialog via Edit → Settings... (⌘,)
  - Supports theme selection (Dark, Light, High Contrast, Auto)
  - Font family selection with 8 different options
  - Font size selection (Small, Medium, Large, Extra Large) 
  - Settings persist across sessions
  - Changes apply immediately without restart

#### **2. Show Hidden Files Toggle (File Menu)**
- **Location:** `src/main.rs:226-245`
- **Functionality:**
  - Toggles visibility of hidden files and folders
  - Persists setting in application preferences
  - Automatically refreshes current directory after toggle
  - State stored in SettingsState.show_hidden_files

#### **3. Refresh Directory (File Menu)**
- **Location:** `src/main.rs:214-225`
- **Functionality:**
  - Refreshes current directory view (⌘R)
  - Reloads file tree contents without changing navigation
  - Handles errors gracefully with logging

#### **4. Clear Selection (Edit Menu)**
- **Location:** `src/main.rs:253-258`
- **Functionality:**
  - Clears all selected files in the file tree (⌘⇧A)
  - Updates selection state immediately
  - Provides visual feedback in UI

#### **5. Keyboard Shortcuts Dialog (Help Menu)**
- **Location:** `src/main.rs:330-335`
- **Functionality:**
  - Opens comprehensive keyboard shortcuts cheat sheet
  - Shows all available shortcuts organized by category
  - Platform-aware display (⌘ on macOS, Ctrl on Windows/Linux)
  - Modal overlay with categorized shortcuts

#### **6. Open Folder... (File Menu)**
- **Location:** `src/main.rs:179-201`
- **Functionality:** 
  - Opens native folder picker dialog using `rfd::AsyncFileDialog`
  - Updates app state with selected folder path
  - Refreshes file tree to show new directory contents
  - Persists folder selection across app restarts
- **Code:**
  ```rust
  "open_folder" => {
      info!("Opening folder selection dialog...");
      let app_state_clone = app_state.clone();
      spawn(async move {
          if let Some(folder_path) = open_folder_dialog().await {
              match app_state_clone.set_root_folder_with_persistence(folder_path.clone()).await {
                  Ok(_) => info!("Successfully changed file tree root to: {:?}", folder_path),
                  Err(e) => info!("Error changing file tree root: {}", e)
              }
          }
      });
  }
  ```

#### **2. Theme Switching (View Menu)**
- **Location:** `src/main.rs:270-302`
- **Functionality:**
  - **Light Theme:** Switches to VSCode Light+ theme
  - **Dark Theme:** Switches to VSCode Dark+ theme  
  - **Auto Theme:** Auto-detects macOS system preference and applies matching theme
  - All theme changes are persisted to settings
  - Uses comprehensive VSCode-style color system with 160+ color tokens
- **Code:**
  ```rust
  "theme_light" => {
      let mut app_state_clone = app_state.clone();
      spawn(async move {
          let mut settings = app_state_clone.settings.write();
          settings.theme = crate::state::Theme::Light;
          crate::theme::ThemeManager::apply_theme(&settings.theme);
          crate::state::save_settings_debounced(settings.clone());
      });
  }
  ```

### Architecture

#### **Menu Creation**
- **Location:** `src/main.rs:42-126`
- **Framework:** Uses `dioxus::desktop::muda` for native menu creation
- **Structure:** Follows macOS Human Interface Guidelines
- **Event Handling:** `use_muda_event_handler` processes menu item clicks

#### **Event Handler**
- **Location:** `src/main.rs:163-297`
- **Pattern:** Match-based event routing with async task spawning
- **Logging:** Comprehensive logging for all menu actions
- **Error Handling:** Graceful error handling with user feedback

#### **Theme System Integration**
- **VSCode Color Themes:** Dark+, Light+, High Contrast variants
- **System Integration:** Cross-platform theme detection
- **CSS Variables:** Auto-generated CSS custom properties
- **Persistence:** Settings saved to local storage with debouncing

---

## Future Enhancement Areas

### **Priority 1: Core File Operations**
- [ ] **New Folder/File Creation** - Essential file management functionality
- [ ] **Delete with Confirmation** - Safe file deletion with trash/permanent options
- [ ] **Rename Inline** - Quick file/folder renaming
- [ ] **Refresh Functionality** - Manual refresh of current directory

### **Priority 2: File Management**
- [ ] **Copy/Move to Dialogs** - Destination picker for file operations
- [ ] **Show in Finder Integration** - Native file manager integration
- [ ] **Open With Menu** - Application picker for file opening
- [ ] **Duplicate Files** - Smart file duplication with naming

### **Priority 3: UI/UX Enhancements**
- [ ] **Settings Dialog** - Comprehensive preferences interface
- [ ] **Sidebar/Panel Toggles** - Layout customization
- [ ] **Hidden Files Toggle** - Show/hide system files
- [ ] **Clear Selection** - Deselect all items

### **Priority 4: Help & Documentation**
- [ ] **Keyboard Shortcuts Dialog** - Interactive shortcut reference
- [ ] **Help Documentation** - In-app help system
- [ ] **About Dialog** - Application information and credits
- [ ] **Update Check System** - Automatic update notifications

---

## Customization Instructions

To modify this menu structure:

1. **Edit this file** with your desired changes
2. **Update menu creation** in `src/main.rs:42-126`
3. **Add event handlers** in `src/main.rs:163-297`
4. **Implement functionality** in appropriate service modules
5. **Update keyboard shortcuts** as needed
6. **Test on macOS** to ensure native behavior

### **Menu Item Format:**
```
├── Menu Item Name               [Keyboard Shortcut]
```

### **Implementation Status:**
- ✅ **IMPLEMENTED** - Feature is working
- ❌ **TODO** - Feature needs implementation
- 🔄 **IN PROGRESS** - Feature is being developed

---

## Technical Notes

### **Keyboard Shortcuts**
- All shortcuts follow macOS conventions
- Modifiers: ⌘ (Cmd), ⌥ (Option), ⇧ (Shift), ⌃ (Control)
- Standard shortcuts (⌘C, ⌘V, etc.) are system-managed

### **Event Handling**
- Async event processing prevents UI blocking
- Error logging for debugging and user feedback
- State management integration for persistence

### **Platform Considerations**
- Menu is currently macOS-specific using `muda`
- Windows/Linux adaptations would need separate implementation
- Cross-platform abstractions available in framework

---

*Last Updated: September 2025*
*Framework: Dioxus 0.6.3 with muda native menus*