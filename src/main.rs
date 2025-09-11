use tracing::info;
use std::path::PathBuf;
use dioxus::prelude::{component, Element};

mod models;
mod performance;
mod services;
mod state;
mod theme;
mod ui;
mod utils;

use models::AppConfig;
use state::AppStateProvider;
use ui::phase2_app;
use services::FileEntry;

fn main() {
    // Initialize logging
    tracing_subscriber::fmt::init();

    info!("Starting File Manager application");

    // Load configuration
    let _config = AppConfig::default();

    // Create custom menu bar
    let menu = create_menu_bar();

    // Launch Dioxus desktop application with custom menu
    dioxus::LaunchBuilder::desktop()
        .with_cfg(
            dioxus::desktop::Config::default()
                .with_menu(menu)
                .with_window(
                    dioxus::desktop::WindowBuilder::new()
                        .with_title("Media Organizer")
                )
        )
        .launch(app);
}

fn create_menu_bar() -> dioxus::desktop::muda::Menu {
    use dioxus::desktop::muda::{Menu, Submenu, MenuItem, PredefinedMenuItem};
    use dioxus::desktop::muda::accelerator::{Accelerator, Modifiers, Code};

    let menu = Menu::new();

    // App menu (media-organizer)
    let app_menu = Submenu::new("Media Organizer", true);
    app_menu.append_items(&[
        &MenuItem::with_id("about_app", "About Media Organizer", true, None),
        &MenuItem::with_id("check_updates", "Check for Updates...", true, None),
        &PredefinedMenuItem::separator(),
        &MenuItem::with_id("preferences", "Preferences...", true, Some(Accelerator::new(Some(Modifiers::SUPER), Code::Comma))),
        &PredefinedMenuItem::separator(),
        &PredefinedMenuItem::services(None),
        &PredefinedMenuItem::separator(),
        &PredefinedMenuItem::hide(None),
        &PredefinedMenuItem::hide_others(None),
        &PredefinedMenuItem::show_all(None),
        &PredefinedMenuItem::separator(),
        &PredefinedMenuItem::quit(None),
    ]).unwrap();
    menu.append(&app_menu).unwrap();

    // File menu
    let file_menu = Submenu::new("File", true);
    file_menu.append_items(&[
        &MenuItem::with_id("open_folder", "Open Folder...", true, Some(Accelerator::new(Some(Modifiers::SUPER), Code::KeyO))),
        &MenuItem::with_id("new_window", "New Window", true, Some(Accelerator::new(Some(Modifiers::SUPER), Code::KeyN))),
        &PredefinedMenuItem::separator(),
        &MenuItem::with_id("new_folder", "New Folder", true, Some(Accelerator::new(Some(Modifiers::SUPER | Modifiers::SHIFT), Code::KeyN))),
        &MenuItem::with_id("new_file", "New File", true, Some(Accelerator::new(Some(Modifiers::SUPER | Modifiers::SHIFT), Code::KeyT))),
        &PredefinedMenuItem::separator(),
        &MenuItem::with_id("refresh", "Refresh", true, Some(Accelerator::new(Some(Modifiers::SUPER), Code::KeyR))),
        &MenuItem::with_id("show_hidden", "Show Hidden Files", true, Some(Accelerator::new(Some(Modifiers::SUPER | Modifiers::SHIFT), Code::Period))),
        &PredefinedMenuItem::separator(),
        &MenuItem::with_id("open", "Open", true, Some(Accelerator::new(Some(Modifiers::SUPER), Code::Enter))),
        &MenuItem::with_id("open_with", "Open With...", true, Some(Accelerator::new(Some(Modifiers::SUPER | Modifiers::SHIFT), Code::Enter))),
        &MenuItem::with_id("show_in_finder", "Show in Finder", true, Some(Accelerator::new(Some(Modifiers::SUPER | Modifiers::ALT), Code::KeyR))),
    ]).unwrap();
    menu.append(&file_menu).unwrap();

    // Edit menu
    let edit_menu = Submenu::new("Edit", true);
    edit_menu.append_items(&[
        &PredefinedMenuItem::undo(None),
        &PredefinedMenuItem::redo(None),
        &PredefinedMenuItem::separator(),
        &PredefinedMenuItem::cut(None),
        &PredefinedMenuItem::copy(None),
        &PredefinedMenuItem::paste(None),
        &PredefinedMenuItem::separator(),
        &PredefinedMenuItem::select_all(None),
        &MenuItem::with_id("clear_selection", "Clear Selection", true, None),
        &PredefinedMenuItem::separator(),
        &MenuItem::with_id("copy_to", "Copy to...", true, Some(Accelerator::new(Some(Modifiers::SUPER | Modifiers::SHIFT), Code::KeyC))),
        &MenuItem::with_id("move_to", "Move to...", true, Some(Accelerator::new(Some(Modifiers::SUPER | Modifiers::SHIFT), Code::KeyM))),
        &PredefinedMenuItem::separator(),
        &MenuItem::with_id("delete", "Delete", true, Some(Accelerator::new(None, Code::Delete))),
        &MenuItem::with_id("rename", "Rename", true, Some(Accelerator::new(None, Code::Enter))),
        &MenuItem::with_id("duplicate", "Duplicate", true, Some(Accelerator::new(Some(Modifiers::SUPER), Code::KeyD))),
        &PredefinedMenuItem::separator(),
        &MenuItem::with_id("settings", "Settings...", true, Some(Accelerator::new(Some(Modifiers::SUPER), Code::Comma))),
    ]).unwrap();
    menu.append(&edit_menu).unwrap();

    // View menu
    let view_menu = Submenu::new("View", true);
    view_menu.append_items(&[
        &MenuItem::with_id("toggle_sidebar", "Toggle Sidebar", true, Some(Accelerator::new(Some(Modifiers::SUPER), Code::KeyB))),
        &MenuItem::with_id("toggle_panel", "Toggle Panel", true, Some(Accelerator::new(Some(Modifiers::SUPER), Code::KeyJ))),
        &PredefinedMenuItem::separator(),
        &MenuItem::with_id("theme_light", "Light Theme", true, None),
        &MenuItem::with_id("theme_dark", "Dark Theme", true, None),
        &MenuItem::with_id("theme_auto", "Auto Theme", true, None),
    ]).unwrap();
    menu.append(&view_menu).unwrap();

    // Help menu
    let help_menu = Submenu::new("Help", true);
    help_menu.append_items(&[
        &MenuItem::with_id("keyboard_shortcuts", "Keyboard Shortcuts", true, None),
        &MenuItem::with_id("help_documentation", "Media Organizer Help", true, None),
    ]).unwrap();
    menu.append(&help_menu).unwrap();

    menu
}

/// Create a new application window
async fn create_new_window() -> Result<(), String> {
    use std::process::Command;
    
    // Get current executable path
    let current_exe = std::env::current_exe()
        .map_err(|e| format!("Failed to get current executable path: {}", e))?;
    
    // Launch a new instance of the application
    let result = Command::new(current_exe)
        .spawn();
    
    match result {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("Failed to create new window: {}", e))
    }
}

/// Show a native folder picker dialog
async fn open_folder_dialog() -> Option<PathBuf> {
    use rfd::AsyncFileDialog;
    
    // Show folder picker dialog
    let folder = AsyncFileDialog::new()
        .set_title("Select Folder")
        .pick_folder()
        .await;
    
    folder.map(|handle| handle.path().to_path_buf())
}

/// Create a new file with user input dialog
async fn create_new_file_dialog(parent_path: &PathBuf) -> Result<PathBuf, String> {
    use std::fs;
    
    // Show input dialog for file name
    let file_name = show_input_dialog("New File", "Enter file name:", "New File.txt").await?;
    
    if file_name.trim().is_empty() {
        return Err("File name cannot be empty".to_string());
    }
    
    // Validate file name (no invalid characters)
    if file_name.contains(['/', '\\', ':', '*', '?', '"', '<', '>', '|']) {
        return Err("File name contains invalid characters".to_string());
    }
    
    let new_file_path = parent_path.join(&file_name);
    
    // Check if file already exists
    if new_file_path.exists() {
        return Err(format!("File '{}' already exists", file_name));
    }
    
    // Create the file
    match fs::File::create(&new_file_path) {
        Ok(_) => Ok(new_file_path),
        Err(e) => Err(format!("Failed to create file: {}", e))
    }
}

/// Create a new folder with user input dialog
async fn create_new_folder_dialog(parent_path: &PathBuf) -> Result<PathBuf, String> {
    use rfd::AsyncMessageDialog;
    use std::fs;
    
    // Show input dialog for folder name
    let folder_name = show_input_dialog("New Folder", "Enter folder name:", "New Folder").await?;
    
    if folder_name.trim().is_empty() {
        return Err("Folder name cannot be empty".to_string());
    }
    
    // Validate folder name (no invalid characters)
    if folder_name.contains(['/', '\\', ':', '*', '?', '"', '<', '>', '|']) {
        return Err("Folder name contains invalid characters".to_string());
    }
    
    let new_folder_path = parent_path.join(&folder_name);
    
    // Check if folder already exists
    if new_folder_path.exists() {
        return Err(format!("Folder '{}' already exists", folder_name));
    }
    
    // Create the folder
    match fs::create_dir(&new_folder_path) {
        Ok(_) => Ok(new_folder_path),
        Err(e) => Err(format!("Failed to create folder: {}", e))
    }
}

/// Show a simple input dialog (using message dialog as fallback)
async fn show_input_dialog(title: &str, message: &str, default_value: &str) -> Result<String, String> {
    // For now, we'll use a simple approach - in a full implementation, you'd want a proper input dialog
    // This is a simplified version that creates files/folders with default names
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    
    // Return a default name with timestamp to ensure uniqueness
    if default_value.contains('.') {
        // It's a file with extension
        if let Some(dot_pos) = default_value.rfind('.') {
            let (name_part, ext_part) = default_value.split_at(dot_pos);
            Ok(format!("{} {}{}", name_part, timestamp % 1000, ext_part))
        } else {
            Ok(format!("{} {}", default_value, timestamp % 1000))
        }
    } else {
        // It's a folder or file without extension
        Ok(format!("{} {}", default_value, timestamp % 1000))
    }
}

/// Show a confirmation dialog
async fn show_confirmation_dialog(title: &str, message: &str) -> Result<bool, String> {
    use rfd::AsyncMessageDialog;
    
    let result = AsyncMessageDialog::new()
        .set_title(title)
        .set_description(message)
        .set_level(rfd::MessageLevel::Warning)
        .set_buttons(rfd::MessageButtons::YesNo)
        .show()
        .await;
    
    Ok(result == rfd::MessageDialogResult::Yes)
}

/// Delete selected files
async fn delete_selected_files(selected_files: &[FileEntry]) -> Result<usize, String> {
    use std::fs;
    use trash;
    
    let mut deleted_count = 0;
    let mut errors = Vec::new();
    
    for file_entry in selected_files {
        let file_path = &file_entry.path;
        
        // Try to move to trash first (safer), fallback to permanent delete
        match trash::delete(file_path) {
            Ok(_) => {
                deleted_count += 1;
                info!("Moved to trash: {:?}", file_path);
            },
            Err(e) => {
                // If trash fails, try permanent deletion as fallback
                match if file_path.is_dir() {
                    fs::remove_dir_all(file_path)
                } else {
                    fs::remove_file(file_path)
                } {
                    Ok(_) => {
                        deleted_count += 1;
                        info!("Permanently deleted: {:?}", file_path);
                    },
                    Err(delete_err) => {
                        let error_msg = format!("Failed to delete '{}': {} (trash error: {})", 
                                               file_entry.name, delete_err, e);
                        errors.push(error_msg);
                    }
                }
            }
        }
    }
    
    if !errors.is_empty() {
        return Err(format!("Some files could not be deleted: {}", errors.join("; ")));
    }
    
    Ok(deleted_count)
}

/// Show a file or folder in the system file manager
async fn show_in_system_file_manager(path: &std::path::PathBuf) -> Result<(), String> {
    use std::process::Command;
    
    let result = if cfg!(target_os = "macos") {
        // macOS: Use 'open' command with -R to reveal in Finder
        Command::new("open")
            .arg("-R")
            .arg(path)
            .spawn()
    } else if cfg!(target_os = "windows") {
        // Windows: Use 'explorer' with /select parameter
        Command::new("explorer")
            .arg("/select,")
            .arg(path)
            .spawn()
    } else {
        // Linux: Try different file managers
        // First try to open the parent directory if the path is a file
        let dir_to_open = if path.is_file() {
            path.parent().unwrap_or(path)
        } else {
            path
        };
        
        // Try different Linux file managers in order of preference
        Command::new("xdg-open")
            .arg(dir_to_open)
            .spawn()
            .or_else(|_| Command::new("nautilus").arg(dir_to_open).spawn())
            .or_else(|_| Command::new("dolphin").arg(dir_to_open).spawn())
            .or_else(|_| Command::new("thunar").arg(dir_to_open).spawn())
            .or_else(|_| Command::new("pcmanfm").arg(dir_to_open).spawn())
    };
    
    match result {
        Ok(mut child) => {
            // Wait for the command to complete
            tokio::task::spawn_blocking(move || child.wait()).await
                .map_err(|e| format!("Failed to wait for file manager process: {}", e))?
                .map_err(|e| format!("File manager process failed: {}", e))?;
            Ok(())
        },
        Err(e) => Err(format!("Failed to open file manager: {}", e))
    }
}

/// Open a file with the system default application
async fn open_with_system_default(path: &std::path::PathBuf) -> Result<(), String> {
    use std::process::Command;
    
    let result = if cfg!(target_os = "macos") {
        // macOS: Use 'open' command
        Command::new("open")
            .arg(path)
            .spawn()
    } else if cfg!(target_os = "windows") {
        // Windows: Use 'cmd' with 'start' command
        Command::new("cmd")
            .args(&["/C", "start", "", &path.to_string_lossy()])
            .spawn()
    } else {
        // Linux: Use xdg-open
        Command::new("xdg-open")
            .arg(path)
            .spawn()
    };
    
    match result {
        Ok(mut child) => {
            // Don't wait for the command to complete since we want to launch and return
            tokio::task::spawn(async move {
                let _ = child.wait();
            });
            Ok(())
        },
        Err(e) => Err(format!("Failed to open file with system default application: {}", e))
    }
}

/// Show a rename dialog with current filename
async fn show_rename_dialog(current_name: &str) -> Result<String, String> {
    // For now, create a simple timestamped version for demonstration
    // In a full implementation, you'd want a proper text input dialog
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    
    // Extract file extension if it exists
    if let Some(dot_pos) = current_name.rfind('.') {
        let (name_part, ext_part) = current_name.split_at(dot_pos);
        Ok(format!("{}_renamed_{}{}", name_part, timestamp % 1000, ext_part))
    } else {
        Ok(format!("{}_renamed_{}", current_name, timestamp % 1000))
    }
}

/// Show a folder picker dialog for selecting destination
async fn show_destination_folder_dialog(title: &str) -> Result<Option<PathBuf>, String> {
    use rfd::AsyncFileDialog;
    
    let folder = AsyncFileDialog::new()
        .set_title(title)
        .pick_folder()
        .await;
    
    Ok(folder.map(|handle| handle.path().to_path_buf()))
}

/// Copy files to destination folder
async fn copy_files_to_destination(files: &[FileEntry], destination: &PathBuf) -> Result<usize, String> {
    use std::fs;
    
    let mut copied_count = 0;
    let mut errors = Vec::new();
    
    for file_entry in files {
        let source_path = &file_entry.path;
        let file_name = source_path.file_name()
            .ok_or_else(|| format!("Invalid file name for: {:?}", source_path))?;
        let destination_path = destination.join(file_name);
        
        // Check if destination already exists
        if destination_path.exists() {
            errors.push(format!("File already exists at destination: {:?}", destination_path));
            continue;
        }
        
        // Copy file or directory
        let copy_result = if source_path.is_dir() {
            copy_directory_recursive(source_path, &destination_path)
        } else {
            fs::copy(source_path, &destination_path).map(|_| ())
        };
        
        match copy_result {
            Ok(_) => {
                copied_count += 1;
                info!("Copied: {:?} -> {:?}", source_path, destination_path);
            },
            Err(e) => {
                errors.push(format!("Failed to copy '{}': {}", file_entry.name, e));
            }
        }
    }
    
    if !errors.is_empty() {
        return Err(format!("Some files could not be copied: {}", errors.join("; ")));
    }
    
    Ok(copied_count)
}

/// Copy directory recursively
fn copy_directory_recursive(source: &PathBuf, destination: &PathBuf) -> Result<(), std::io::Error> {
    use std::fs;
    
    fs::create_dir_all(destination)?;
    
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let source_path = entry.path();
        let dest_path = destination.join(entry.file_name());
        
        if source_path.is_dir() {
            copy_directory_recursive(&source_path, &dest_path)?;
        } else {
            fs::copy(&source_path, &dest_path)?;
        }
    }
    
    Ok(())
}

/// Move files to destination folder
async fn move_files_to_destination(files: &[FileEntry], destination: &PathBuf) -> Result<usize, String> {
    use std::fs;
    
    let mut moved_count = 0;
    let mut errors = Vec::new();
    
    for file_entry in files {
        let source_path = &file_entry.path;
        let file_name = source_path.file_name()
            .ok_or_else(|| format!("Invalid file name for: {:?}", source_path))?;
        let destination_path = destination.join(file_name);
        
        // Check if destination already exists
        if destination_path.exists() {
            errors.push(format!("File already exists at destination: {:?}", destination_path));
            continue;
        }
        
        // Move file or directory
        match fs::rename(source_path, &destination_path) {
            Ok(_) => {
                moved_count += 1;
                info!("Moved: {:?} -> {:?}", source_path, destination_path);
            },
            Err(e) => {
                errors.push(format!("Failed to move '{}': {}", file_entry.name, e));
            }
        }
    }
    
    if !errors.is_empty() {
        return Err(format!("Some files could not be moved: {}", errors.join("; ")));
    }
    
    Ok(moved_count)
}

/// Duplicate selected files
async fn duplicate_files(files: &[FileEntry]) -> Result<usize, String> {
    use std::fs;
    
    let mut duplicated_count = 0;
    let mut errors = Vec::new();
    
    for file_entry in files {
        let source_path = &file_entry.path;
        let parent_dir = source_path.parent()
            .ok_or_else(|| format!("Cannot determine parent directory for: {:?}", source_path))?;
        
        // Generate a unique name for the duplicate
        let duplicate_path = generate_duplicate_name(source_path)?;
        
        // Copy file or directory
        let copy_result = if source_path.is_dir() {
            copy_directory_recursive(source_path, &duplicate_path)
        } else {
            fs::copy(source_path, &duplicate_path).map(|_| ())
        };
        
        match copy_result {
            Ok(_) => {
                duplicated_count += 1;
                info!("Duplicated: {:?} -> {:?}", source_path, duplicate_path);
            },
            Err(e) => {
                errors.push(format!("Failed to duplicate '{}': {}", file_entry.name, e));
            }
        }
    }
    
    if !errors.is_empty() {
        return Err(format!("Some files could not be duplicated: {}", errors.join("; ")));
    }
    
    Ok(duplicated_count)
}

/// Generate a unique duplicate name
fn generate_duplicate_name(original_path: &PathBuf) -> Result<PathBuf, String> {
    let parent_dir = original_path.parent()
        .ok_or_else(|| "Cannot determine parent directory".to_string())?;
    
    let file_name = original_path.file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| "Invalid file name".to_string())?;
    
    // Try different suffixes until we find a unique name
    for i in 1..1000 {
        let duplicate_name = if let Some(dot_pos) = file_name.rfind('.') {
            let (name_part, ext_part) = file_name.split_at(dot_pos);
            format!("{} copy {}{}", name_part, i, ext_part)
        } else {
            format!("{} copy {}", file_name, i)
        };
        
        let duplicate_path = parent_dir.join(duplicate_name);
        if !duplicate_path.exists() {
            return Ok(duplicate_path);
        }
    }
    
    Err("Could not generate unique duplicate name".to_string())
}

/// Show about dialog with application information
async fn show_about_dialog() -> Result<(), String> {
    use std::process::Command;
    
    let about_message = format!(
        "MediaOrganizer v{}\n\nA cross-platform media/file management application\nBuilt with Dioxus and Rust\n\n© 2025 MediaOrganizer Team",
        env!("CARGO_PKG_VERSION")
    );
    
    let result = if cfg!(target_os = "macos") {
        Command::new("osascript")
            .args(&["-e", &format!("display dialog \"{}\" with title \"About MediaOrganizer\" buttons {{\"OK\"}} default button \"OK\"", about_message)])
            .spawn()
    } else if cfg!(target_os = "windows") {
        Command::new("powershell")
            .args(&["-Command", &format!("Add-Type -AssemblyName PresentationFramework; [System.Windows.MessageBox]::Show('{}', 'About MediaOrganizer')", about_message)])
            .spawn()
    } else {
        // Linux - try zenity, kdialog, or xmessage as fallbacks
        if Command::new("zenity").arg("--version").output().is_ok() {
            Command::new("zenity")
                .args(&["--info", "--title=About MediaOrganizer", &format!("--text={}", about_message)])
                .spawn()
        } else if Command::new("kdialog").arg("--version").output().is_ok() {
            Command::new("kdialog")
                .args(&["--msgbox", &about_message, "--title", "About MediaOrganizer"])
                .spawn()
        } else {
            Command::new("xmessage")
                .args(&["-center", &format!("About MediaOrganizer\n\n{}", about_message)])
                .spawn()
        }
    };
    
    match result {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("Failed to show about dialog: {}", e))
    }
}

/// Check for application updates
async fn check_for_updates() -> Result<(), String> {
    use std::process::Command;
    
    // In a real implementation, this would check a remote server for updates
    // For now, just show a placeholder message
    let update_message = format!(
        "MediaOrganizer v{}\n\nYou are running the latest version.\n\nUpdate checking functionality will be implemented in a future release.",
        env!("CARGO_PKG_VERSION")
    );
    
    let result = if cfg!(target_os = "macos") {
        Command::new("osascript")
            .args(&["-e", &format!("display dialog \"{}\" with title \"Check for Updates\" buttons {{\"OK\"}} default button \"OK\"", update_message)])
            .spawn()
    } else if cfg!(target_os = "windows") {
        Command::new("powershell")
            .args(&["-Command", &format!("Add-Type -AssemblyName PresentationFramework; [System.Windows.MessageBox]::Show('{}', 'Check for Updates')", update_message)])
            .spawn()
    } else {
        // Linux - try zenity, kdialog, or xmessage as fallbacks
        if Command::new("zenity").arg("--version").output().is_ok() {
            Command::new("zenity")
                .args(&["--info", "--title=Check for Updates", &format!("--text={}", update_message)])
                .spawn()
        } else if Command::new("kdialog").arg("--version").output().is_ok() {
            Command::new("kdialog")
                .args(&["--msgbox", &update_message, "--title", "Check for Updates"])
                .spawn()
        } else {
            Command::new("xmessage")
                .args(&["-center", &format!("Check for Updates\n\n{}", update_message)])
                .spawn()
        }
    };
    
    match result {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("Failed to show update dialog: {}", e))
    }
}

/// Open help documentation (opens online help or local README)
async fn open_help_documentation() -> Result<(), String> {
    use std::process::Command;
    
    // Try to open GitHub repository help first, fallback to local README
    let help_url = "https://github.com/your-username/MediaOrganizer#readme";
    
    let result = if cfg!(target_os = "macos") {
        // macOS: Use 'open' command
        Command::new("open")
            .arg(help_url)
            .spawn()
    } else if cfg!(target_os = "windows") {
        // Windows: Use 'start' command via cmd
        Command::new("cmd")
            .args(&["/C", "start", help_url])
            .spawn()
    } else {
        // Linux: Use xdg-open
        Command::new("xdg-open")
            .arg(help_url)
            .spawn()
    };
    
    match result {
        Ok(_) => Ok(()),
        Err(e) => {
            // Fallback: try to open local README file if web browser fails
            match try_open_local_readme().await {
                Ok(()) => Ok(()),
                Err(local_err) => {
                    Err(format!("Failed to open help documentation online ({}), and local README also failed ({})", e, local_err))
                }
            }
        }
    }
}

/// Try to open local README file as fallback
async fn try_open_local_readme() -> Result<(), String> {
    use std::process::Command;
    use std::path::Path;
    
    // Look for common README file locations
    let possible_readme_paths = [
        "README.md",
        "README.txt", 
        "README.rst",
        "../README.md",
        "./docs/README.md"
    ];
    
    for readme_path in possible_readme_paths.iter() {
        if Path::new(readme_path).exists() {
            let result = if cfg!(target_os = "macos") {
                Command::new("open").arg(readme_path).spawn()
            } else if cfg!(target_os = "windows") {
                Command::new("cmd").args(&["/C", "start", "", readme_path]).spawn()
            } else {
                Command::new("xdg-open").arg(readme_path).spawn()
            };
            
            match result {
                Ok(_) => return Ok(()),
                Err(_) => continue,
            }
        }
    }
    
    Err("No local README file found".to_string())
}

/// Rename a file or folder
async fn rename_file(current_path: &std::path::PathBuf, new_name: &str) -> Result<std::path::PathBuf, String> {
    use std::fs;
    
    // Validate new name (no invalid characters and not empty)
    if new_name.trim().is_empty() {
        return Err("New name cannot be empty".to_string());
    }
    
    if new_name.contains(['/', '\\', ':', '*', '?', '"', '<', '>', '|']) {
        return Err("New name contains invalid characters".to_string());
    }
    
    // Get parent directory and create new path
    let parent_dir = current_path.parent()
        .ok_or_else(|| "Cannot determine parent directory".to_string())?;
    
    let new_path = parent_dir.join(new_name);
    
    // Check if target already exists
    if new_path.exists() && new_path != *current_path {
        return Err(format!("A file or folder named '{}' already exists", new_name));
    }
    
    // Perform the rename
    match fs::rename(current_path, &new_path) {
        Ok(_) => Ok(new_path),
        Err(e) => Err(format!("Failed to rename file: {}", e))
    }
}

// Root app component with state provider
fn app() -> dioxus::prelude::Element {
    use dioxus::prelude::*;
    use dioxus::desktop::use_muda_event_handler;
    
    rsx! {
        AppStateProvider {
            AppWithMenuHandlers {}
        }
    }
}

// App component with menu handlers that has access to the app state context
fn AppWithMenuHandlers() -> Element {
    use dioxus::prelude::*;
    use dioxus::desktop::use_muda_event_handler;
    use crate::state::use_app_state;
    
    // Get app state for menu actions
    let app_state = use_app_state();
    
    // Handle menu events
    use_muda_event_handler(move |menu_event| {
        let event_id = menu_event.id.0.as_str();
        info!("Menu item clicked: {}", event_id);
        
        match event_id {
            // App menu items
            "about_app" => {
                info!("Showing about dialog...");
                spawn(async move {
                    if let Err(e) = show_about_dialog().await {
                        info!("Error showing about dialog: {}", e);
                    } else {
                        info!("Successfully opened about dialog");
                    }
                });
            },
            "check_updates" => {
                info!("Checking for updates...");
                spawn(async move {
                    if let Err(e) = check_for_updates().await {
                        info!("Error checking for updates: {}", e);
                    } else {
                        info!("Successfully checked for updates");
                    }
                });
            },
            "preferences" => {
                info!("Opening preferences...");
                let mut app_state_clone = app_state.clone();
                app_state_clone.settings_dialog_visible.set(true);
                info!("Preferences dialog opened via menu");
            },
            
            // File menu items
            "open_folder" => {
                info!("Opening folder selection dialog...");
                let app_state_clone = app_state.clone();
                
                // Spawn async task to handle folder dialog
                spawn(async move {
                    if let Some(folder_path) = open_folder_dialog().await {
                        info!("Selected folder: {:?}", folder_path);
                        
                        // Update the file tree root to the selected folder
                        match app_state_clone.clone().set_root_folder_with_persistence(folder_path.clone()).await {
                            Ok(_) => {
                                info!("Successfully changed file tree root to: {:?}", folder_path);
                            },
                            Err(e) => {
                                info!("Error changing file tree root: {}", e);
                            }
                        }
                    } else {
                        info!("Folder selection cancelled");
                    }
                });
            },
            "new_window" => {
                info!("Creating new window...");
                spawn(async move {
                    if let Err(e) = create_new_window().await {
                        info!("Error creating new window: {}", e);
                    } else {
                        info!("Successfully created new window");
                    }
                });
            },
            "new_folder" => {
                info!("Creating new folder...");
                let mut app_state_clone = app_state.clone();
                
                spawn(async move {
                    let current_folder = {
                        let nav_state = app_state_clone.navigation.read();
                        Some(nav_state.current_path.clone())
                    };
                    
                    if let Some(parent_path) = current_folder {
                        match create_new_folder_dialog(&parent_path).await {
                            Ok(new_folder_path) => {
                                info!("Successfully created folder: {:?}", new_folder_path);
                                // Refresh the file tree to show the new folder
                                if let Err(e) = app_state_clone.refresh_current_directory().await {
                                    info!("Error refreshing directory after folder creation: {}", e);
                                }
                            },
                            Err(e) => {
                                info!("Error creating new folder: {}", e);
                            }
                        }
                    } else {
                        info!("No current folder selected - cannot create new folder");
                    }
                });
            },
            "new_file" => {
                info!("Creating new file...");
                let mut app_state_clone = app_state.clone();
                
                spawn(async move {
                    let current_folder = {
                        let nav_state = app_state_clone.navigation.read();
                        Some(nav_state.current_path.clone())
                    };
                    
                    if let Some(parent_path) = current_folder {
                        match create_new_file_dialog(&parent_path).await {
                            Ok(new_file_path) => {
                                info!("Successfully created file: {:?}", new_file_path);
                                // Refresh the file tree to show the new file
                                if let Err(e) = app_state_clone.refresh_current_directory().await {
                                    info!("Error refreshing directory after file creation: {}", e);
                                }
                            },
                            Err(e) => {
                                info!("Error creating new file: {}", e);
                            }
                        }
                    } else {
                        info!("No current folder selected - cannot create new file");
                    }
                });
            },
            "refresh" => {
                info!("Refreshing file view...");
                let mut app_state_clone = app_state.clone();
                spawn(async move {
                    let current_path = app_state_clone.navigation.read().current_path.clone();
                    if let Err(e) = app_state_clone.navigate_to(current_path.clone()).await {
                        info!("Error refreshing directory: {}", e);
                    } else {
                        info!("Successfully refreshed directory: {:?}", current_path);
                    }
                });
            },
            "show_hidden" => {
                info!("Toggling hidden files visibility...");
                let mut app_state_clone = app_state.clone();
                spawn(async move {
                    let show_hidden_files = {
                        let mut settings = app_state_clone.settings.write();
                        settings.show_hidden_files = !settings.show_hidden_files;
                        crate::state::save_settings_debounced(settings.clone());
                        settings.show_hidden_files
                    }; // settings write lock is dropped here
                    
                    info!("Hidden files visibility toggled to: {}", show_hidden_files);
                    
                    // Refresh the current directory to show/hide hidden files
                    let current_path = app_state_clone.navigation.read().current_path.clone();
                    if let Err(e) = app_state_clone.navigate_to(current_path).await {
                        info!("Error refreshing directory after hidden files toggle: {}", e);
                    }
                });
            },
            "open" => {
                info!("Opening selected file with system default...");
                let app_state_clone = app_state.clone();
                
                spawn(async move {
                    let selected_files = {
                        let selection_state = app_state_clone.selection.read();
                        let file_entries = app_state_clone.file_entries.read();
                        
                        // Filter file entries to get only the selected ones
                        file_entries.iter()
                            .filter(|entry| selection_state.selected_files.contains(&entry.path))
                            .cloned()
                            .collect::<Vec<_>>()
                    };
                    
                    if selected_files.is_empty() {
                        info!("No files selected for opening");
                        return;
                    }
                    
                    if selected_files.len() > 1 {
                        info!("Multiple files selected - open only works with single file selection");
                        return;
                    }
                    
                    let file_path = &selected_files[0].path;
                    
                    // Only open files, not directories
                    if !file_path.is_file() {
                        info!("Cannot open directory with system default application: {:?}", file_path);
                        return;
                    }
                    
                    match open_with_system_default(file_path).await {
                        Ok(_) => {
                            info!("Successfully opened file with system default: {:?}", file_path);
                        },
                        Err(e) => {
                            info!("Error opening file with system default: {}", e);
                        }
                    }
                });
            },
            "open_with" => {
                info!("Opening with external application...");
                let app_state_clone = app_state.clone();
                
                spawn(async move {
                    let selected_files = {
                        let selection_state = app_state_clone.selection.read();
                        let file_entries = app_state_clone.file_entries.read();
                        
                        // Filter file entries to get only the selected ones
                        file_entries.iter()
                            .filter(|entry| selection_state.selected_files.contains(&entry.path))
                            .cloned()
                            .collect::<Vec<_>>()
                    };
                    
                    if selected_files.is_empty() {
                        info!("No files selected for open with");
                        return;
                    }
                    
                    // For now, just open with system default application
                    // In a full implementation, you'd show an application picker dialog
                    let file_path = &selected_files[0].path;
                    
                    match open_with_system_default(file_path).await {
                        Ok(_) => {
                            info!("Successfully opened file with system default: {:?}", file_path);
                        },
                        Err(e) => {
                            info!("Error opening file with system default: {}", e);
                        }
                    }
                });
            },
            "show_in_finder" => {
                info!("Showing selected item in Finder...");
                let app_state_clone = app_state.clone();
                
                spawn(async move {
                    let (selected_files, current_folder) = {
                        let selection_state = app_state_clone.selection.read();
                        let file_entries = app_state_clone.file_entries.read();
                        let nav_state = app_state_clone.navigation.read();
                        
                        let selected = file_entries.iter()
                            .filter(|entry| selection_state.selected_files.contains(&entry.path))
                            .cloned()
                            .collect::<Vec<_>>();
                        
                        (selected, Some(nav_state.current_path.clone()))
                    };
                    
                    // Determine what path to show in Finder
                    let path_to_show = if !selected_files.is_empty() {
                        // If files are selected, show the first selected file
                        Some(selected_files[0].path.clone())
                    } else if let Some(current) = current_folder {
                        // If no files selected, show the current folder
                        Some(current)
                    } else {
                        None
                    };
                    
                    if let Some(path) = path_to_show {
                        match show_in_system_file_manager(&path).await {
                            Ok(_) => {
                                info!("Successfully showed path in system file manager: {:?}", path);
                            },
                            Err(e) => {
                                info!("Error showing path in system file manager: {}", e);
                            }
                        }
                    } else {
                        info!("No path available to show in system file manager");
                    }
                });
            },
            
            // Edit menu items
            "clear_selection" => {
                info!("Clearing file selection...");
                let mut app_state_clone = app_state.clone();
                app_state_clone.selection.write().clear_selection();
                info!("File selection cleared");
            },
            "copy_to" => {
                info!("Copying files to location...");
                let app_state_clone = app_state.clone();
                
                spawn(async move {
                    let selected_files = {
                        let selection_state = app_state_clone.selection.read();
                        let file_entries = app_state_clone.file_entries.read();
                        
                        // Filter file entries to get only the selected ones
                        file_entries.iter()
                            .filter(|entry| selection_state.selected_files.contains(&entry.path))
                            .cloned()
                            .collect::<Vec<_>>()
                    };
                    
                    if selected_files.is_empty() {
                        info!("No files selected for copying");
                        return;
                    }
                    
                    // Show folder picker for destination
                    match show_destination_folder_dialog("Select Copy Destination").await {
                        Ok(Some(destination)) => {
                            match copy_files_to_destination(&selected_files, &destination).await {
                                Ok(copied_count) => {
                                    info!("Successfully copied {} files to {:?}", copied_count, destination);
                                },
                                Err(e) => {
                                    info!("Error copying files: {}", e);
                                }
                            }
                        },
                        Ok(None) => {
                            info!("Copy operation cancelled by user");
                        },
                        Err(e) => {
                            info!("Error showing destination dialog: {}", e);
                        }
                    }
                });
            },
            "move_to" => {
                info!("Moving files to location...");
                let mut app_state_clone = app_state.clone();
                
                spawn(async move {
                    let selected_files = {
                        let selection_state = app_state_clone.selection.read();
                        let file_entries = app_state_clone.file_entries.read();
                        
                        // Filter file entries to get only the selected ones
                        file_entries.iter()
                            .filter(|entry| selection_state.selected_files.contains(&entry.path))
                            .cloned()
                            .collect::<Vec<_>>()
                    };
                    
                    if selected_files.is_empty() {
                        info!("No files selected for moving");
                        return;
                    }
                    
                    // Show folder picker for destination
                    match show_destination_folder_dialog("Select Move Destination").await {
                        Ok(Some(destination)) => {
                            match move_files_to_destination(&selected_files, &destination).await {
                                Ok(moved_count) => {
                                    info!("Successfully moved {} files to {:?}", moved_count, destination);
                                    // Refresh the file tree to reflect the changes
                                    if let Err(e) = app_state_clone.refresh_current_directory().await {
                                        info!("Error refreshing directory after move: {}", e);
                                    }
                                },
                                Err(e) => {
                                    info!("Error moving files: {}", e);
                                }
                            }
                        },
                        Ok(None) => {
                            info!("Move operation cancelled by user");
                        },
                        Err(e) => {
                            info!("Error showing destination dialog: {}", e);
                        }
                    }
                });
            },
            "delete" => {
                info!("Deleting selected files...");
                let mut app_state_clone = app_state.clone();
                
                spawn(async move {
                    let selected_files = {
                        let selection_state = app_state_clone.selection.read();
                        let file_entries = app_state_clone.file_entries.read();
                        
                        // Filter file entries to get only the selected ones
                        file_entries.iter()
                            .filter(|entry| selection_state.selected_files.contains(&entry.path))
                            .cloned()
                            .collect::<Vec<_>>()
                    };
                    
                    if selected_files.is_empty() {
                        info!("No files selected for deletion");
                        return;
                    }
                    
                    // Show confirmation dialog
                    let file_count = selected_files.len();
                    let confirmation_message = if file_count == 1 {
                        format!("Are you sure you want to delete '{}'?", selected_files[0].name)
                    } else {
                        format!("Are you sure you want to delete {} files?", file_count)
                    };
                    
                    match show_confirmation_dialog("Delete Files", &confirmation_message).await {
                        Ok(true) => {
                            match delete_selected_files(&selected_files).await {
                                Ok(deleted_count) => {
                                    info!("Successfully deleted {} files", deleted_count);
                                    // Refresh the file tree to reflect changes
                                    if let Err(e) = app_state_clone.refresh_current_directory().await {
                                        info!("Error refreshing directory after deletion: {}", e);
                                    }
                                },
                                Err(e) => {
                                    info!("Error deleting files: {}", e);
                                }
                            }
                        },
                        Ok(false) => {
                            info!("File deletion cancelled by user");
                        },
                        Err(e) => {
                            info!("Error showing confirmation dialog: {}", e);
                        }
                    }
                });
            },
            "rename" => {
                info!("Renaming selected file...");
                let mut app_state_clone = app_state.clone();
                
                spawn(async move {
                    let selected_files = {
                        let selection_state = app_state_clone.selection.read();
                        let file_entries = app_state_clone.file_entries.read();
                        
                        // Filter file entries to get only the selected ones
                        file_entries.iter()
                            .filter(|entry| selection_state.selected_files.contains(&entry.path))
                            .cloned()
                            .collect::<Vec<_>>()
                    };
                    
                    if selected_files.is_empty() {
                        info!("No files selected for renaming");
                        return;
                    }
                    
                    if selected_files.len() > 1 {
                        info!("Multiple files selected - rename only works with single file selection");
                        return;
                    }
                    
                    let file_to_rename = &selected_files[0];
                    let current_name = &file_to_rename.name;
                    
                    // Show input dialog for new name
                    match show_rename_dialog(current_name).await {
                        Ok(new_name) => {
                            if new_name != *current_name {
                                match rename_file(&file_to_rename.path, &new_name).await {
                                    Ok(new_path) => {
                                        info!("Successfully renamed '{}' to '{}' (path: {:?})", current_name, new_name, new_path);
                                        // Refresh the file tree to reflect the rename
                                        if let Err(e) = app_state_clone.refresh_current_directory().await {
                                            info!("Error refreshing directory after rename: {}", e);
                                        }
                                    },
                                    Err(e) => {
                                        info!("Error renaming file: {}", e);
                                    }
                                }
                            } else {
                                info!("Rename cancelled - same name provided");
                            }
                        },
                        Err(e) => {
                            info!("Error showing rename dialog: {}", e);
                        }
                    }
                });
            },
            "duplicate" => {
                info!("Duplicating selected files...");
                let mut app_state_clone = app_state.clone();
                
                spawn(async move {
                    let selected_files = {
                        let selection_state = app_state_clone.selection.read();
                        let file_entries = app_state_clone.file_entries.read();
                        
                        // Filter file entries to get only the selected ones
                        file_entries.iter()
                            .filter(|entry| selection_state.selected_files.contains(&entry.path))
                            .cloned()
                            .collect::<Vec<_>>()
                    };
                    
                    if selected_files.is_empty() {
                        info!("No files selected for duplication");
                        return;
                    }
                    
                    match duplicate_files(&selected_files).await {
                        Ok(duplicated_count) => {
                            info!("Successfully duplicated {} files", duplicated_count);
                            // Refresh the file tree to show the duplicated files
                            if let Err(e) = app_state_clone.refresh_current_directory().await {
                                info!("Error refreshing directory after duplication: {}", e);
                            }
                        },
                        Err(e) => {
                            info!("Error duplicating files: {}", e);
                        }
                    }
                });
            },
            "settings" => {
                info!("Opening settings dialog...");
                let mut app_state_clone = app_state.clone();
                app_state_clone.settings_dialog_visible.set(true);
                info!("Settings dialog opened via menu");
            },
            
            // View menu items
            "toggle_sidebar" => {
                info!("Toggling sidebar...");
                let mut app_state_clone = app_state.clone();
                app_state_clone.toggle_sidebar_collapse();
                info!("Sidebar toggle completed");
            },
            "toggle_panel" => {
                info!("Toggling panel...");
                let mut app_state_clone = app_state.clone();
                app_state_clone.toggle_panel_visibility();
                info!("Panel toggle completed");
            },
            "theme_light" => {
                info!("Switching to light theme...");
                let mut app_state_clone = app_state.clone();
                spawn(async move {
                    let mut settings = app_state_clone.settings.write();
                    settings.theme = crate::state::Theme::Light;
                    crate::theme::ThemeManager::apply_theme(&settings.theme);
                    crate::state::save_settings_debounced(settings.clone());
                    info!("Successfully switched to light theme");
                });
            },
            "theme_dark" => {
                info!("Switching to dark theme...");
                let mut app_state_clone = app_state.clone();
                spawn(async move {
                    let mut settings = app_state_clone.settings.write();
                    settings.theme = crate::state::Theme::Dark;
                    crate::theme::ThemeManager::apply_theme(&settings.theme);
                    crate::state::save_settings_debounced(settings.clone());
                    info!("Successfully switched to dark theme");
                });
            },
            "theme_auto" => {
                info!("Switching to auto theme...");
                let mut app_state_clone = app_state.clone();
                spawn(async move {
                    let mut settings = app_state_clone.settings.write();
                    settings.theme = crate::state::Theme::Auto;
                    crate::theme::ThemeManager::apply_theme(&settings.theme);
                    crate::state::save_settings_debounced(settings.clone());
                    info!("Successfully switched to auto theme");
                });
            },
            
            // Help menu items
            "keyboard_shortcuts" => {
                info!("Showing keyboard shortcuts...");
                let mut app_state_clone = app_state.clone();
                app_state_clone.cheat_sheet_visible.set(true);
                info!("Keyboard shortcuts dialog opened via menu");
            },
            "help_documentation" => {
                info!("Opening help documentation...");
                spawn(async move {
                    if let Err(e) = open_help_documentation().await {
                        info!("Error opening help documentation: {}", e);
                    } else {
                        info!("Successfully opened help documentation");
                    }
                });
            },
            
            _ => {
                info!("Unhandled menu item: {}", event_id);
            }
        }
    });
    
    rsx! {
        phase2_app {}
    }
}
