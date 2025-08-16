use dioxus::prelude::*;
use std::path::PathBuf;
use crate::services::FileEntry;
use crate::state::{use_app_state, use_selection_state};

/// Drag and drop operation types
#[derive(Debug, Clone, PartialEq)]
pub enum DragOperation {
    Move,
    Copy,
    Link,
}

impl DragOperation {
    pub fn from_modifiers(ctrl: bool, shift: bool, alt: bool) -> Self {
        if ctrl && !shift {
            DragOperation::Copy
        } else if alt {
            DragOperation::Link
        } else {
            DragOperation::Move
        }
    }

    pub fn cursor_style(&self) -> &'static str {
        match self {
            DragOperation::Move => "move",
            DragOperation::Copy => "copy",
            DragOperation::Link => "alias",
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            DragOperation::Move => "↗️",
            DragOperation::Copy => "📋",
            DragOperation::Link => "🔗",
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            DragOperation::Move => "Move",
            DragOperation::Copy => "Copy",
            DragOperation::Link => "Link",
        }
    }
}

/// Drag state information
#[derive(Debug, Clone)]
pub struct DragState {
    pub is_dragging: bool,
    pub drag_files: Vec<FileEntry>,
    pub operation: DragOperation,
    pub start_position: (f64, f64),
    pub current_position: (f64, f64),
    pub drag_preview_visible: bool,
}

impl Default for DragState {
    fn default() -> Self {
        Self {
            is_dragging: false,
            drag_files: Vec::new(),
            operation: DragOperation::Move,
            start_position: (0.0, 0.0),
            current_position: (0.0, 0.0),
            drag_preview_visible: false,
        }
    }
}

impl DragState {
    pub fn start_drag(&mut self, files: Vec<FileEntry>, start_x: f64, start_y: f64, operation: DragOperation) {
        self.is_dragging = true;
        self.drag_files = files;
        self.operation = operation.clone();
        self.start_position = (start_x, start_y);
        self.current_position = (start_x, start_y);
        self.drag_preview_visible = true;
        
        tracing::info!("Started drag operation: {:?} with {} files", operation, self.drag_files.len());
    }

    pub fn update_position(&mut self, x: f64, y: f64) {
        if self.is_dragging {
            self.current_position = (x, y);
        }
    }

    pub fn update_operation(&mut self, operation: DragOperation) {
        if self.is_dragging {
            self.operation = operation;
        }
    }

    pub fn end_drag(&mut self) {
        self.is_dragging = false;
        self.drag_files.clear();
        self.drag_preview_visible = false;
        
        tracing::info!("Ended drag operation");
    }

    pub fn get_drag_distance(&self) -> f64 {
        let dx = self.current_position.0 - self.start_position.0;
        let dy = self.current_position.1 - self.start_position.1;
        (dx * dx + dy * dy).sqrt()
    }
}

/// Drop zone state
#[derive(Debug, Clone, PartialEq)]
pub enum DropZoneState {
    Idle,
    DragOver,
    DragOverValid,
    DragOverInvalid,
}

impl DropZoneState {
    pub fn style_class(&self) -> &'static str {
        match self {
            DropZoneState::Idle => "drop-zone-idle",
            DropZoneState::DragOver => "drop-zone-drag-over",
            DropZoneState::DragOverValid => "drop-zone-valid",
            DropZoneState::DragOverInvalid => "drop-zone-invalid",
        }
    }

    pub fn border_style(&self) -> &'static str {
        match self {
            DropZoneState::Idle => "border: 1px solid transparent;",
            DropZoneState::DragOver => "border: 2px dashed #007acc;",
            DropZoneState::DragOverValid => "border: 2px solid #28a745; background-color: rgba(40, 167, 69, 0.1);",
            DropZoneState::DragOverInvalid => "border: 2px solid #dc3545; background-color: rgba(220, 53, 69, 0.1);",
        }
    }
}

/// Props for drag preview component
#[derive(Props, Clone, PartialEq)]
pub struct DragPreviewProps {
    pub drag_state: Signal<DragState>,
}

/// Drag preview component that follows the cursor
#[component]
pub fn DragPreview(props: DragPreviewProps) -> Element {
    let drag_state = props.drag_state.read();
    
    if !drag_state.drag_preview_visible || drag_state.drag_files.is_empty() {
        return rsx! { div {} };
    }

    let preview_style = format!(
        "position: fixed; left: {}px; top: {}px; z-index: 10000; 
         pointer-events: none; background: rgba(255, 255, 255, 0.9); 
         border: 1px solid #ccc; border-radius: 4px; padding: 8px; 
         box-shadow: 0 2px 8px rgba(0,0,0,0.2); font-size: 12px; 
         max-width: 300px; cursor: {};",
        drag_state.current_position.0 + 10.0, // Offset from cursor
        drag_state.current_position.1 + 10.0,
        drag_state.operation.cursor_style()
    );

    let file_count = drag_state.drag_files.len();
    let first_file = &drag_state.drag_files[0];

    rsx! {
        div {
            style: "{preview_style}",
            
            div {
                style: "display: flex; align-items: center; gap: 8px; margin-bottom: 4px;",
                span { style: "font-size: 16px;", "{drag_state.operation.icon()}" }
                span { style: "font-weight: bold;", "{drag_state.operation.label()}" }
            }
            
            div {
                style: "display: flex; align-items: center; gap: 8px;",
                span { 
                    style: "font-size: 16px;",
                    if first_file.is_directory { "📁" } else { "📄" }
                }
                span { 
                    style: "max-width: 200px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap;",
                    "{first_file.name}"
                }
            }
            
            {if file_count > 1 {
                rsx! {
                    div {
                        style: "margin-top: 4px; color: #666; font-size: 11px;",
                        {format!("and {} more item{}", file_count - 1, if file_count > 2 { "s" } else { "" })}
                    }
                }
            } else {
                rsx! { div {} }
            }}
        }
    }
}

/// Props for drop zone component
#[derive(Props, Clone, PartialEq)]
pub struct DropZoneProps {
    pub drop_state: Signal<DropZoneState>,
    pub target_path: Option<PathBuf>,
    pub on_drop: EventHandler<(Vec<FileEntry>, DragOperation, PathBuf)>,
    pub children: Element,
}

/// Drop zone component that can receive dragged files
#[component]
pub fn DropZone(mut props: DropZoneProps) -> Element {
    let drop_state = props.drop_state.read();
    let _app_state = use_app_state();
    
    let base_style = format!(
        "position: relative; {}",
        drop_state.border_style()
    );

    rsx! {
        div {
            style: "{base_style}",
            class: "{drop_state.style_class()}",
            
            ondragover: move |e| {
                e.prevent_default();
                props.drop_state.set(DropZoneState::DragOverValid);
            },
            
            ondragenter: move |e| {
                e.prevent_default();
                props.drop_state.set(DropZoneState::DragOver);
            },
            
            ondragleave: move |e| {
                e.prevent_default();
                props.drop_state.set(DropZoneState::Idle);
            },
            
            ondrop: move |e| {
                e.prevent_default();
                props.drop_state.set(DropZoneState::Idle);
                
                // TODO: Handle actual drop event
                // For now, this is a placeholder for when we implement
                // the full drag and drop data transfer
                tracing::info!("Drop event received on target: {:?}", props.target_path);
            },
            
            {props.children}
        }
    }
}

/// Hook to manage drag and drop state
pub fn use_drag_drop() -> (Signal<DragState>, impl FnMut(Vec<FileEntry>, f64, f64, DragOperation), impl FnMut(f64, f64), impl FnMut()) {
    let drag_state = use_signal(DragState::default);
    
    let start_drag = {
        let mut drag_state = drag_state.clone();
        move |files: Vec<FileEntry>, x: f64, y: f64, operation: DragOperation| {
            drag_state.write().start_drag(files, x, y, operation);
        }
    };
    
    let update_drag = {
        let mut drag_state = drag_state.clone();
        move |x: f64, y: f64| {
            drag_state.write().update_position(x, y);
        }
    };
    
    let end_drag = {
        let mut drag_state = drag_state.clone();
        move || {
            drag_state.write().end_drag();
        }
    };
    
    (drag_state, start_drag, update_drag, end_drag)
}

/// Hook to manage drop zone state
pub fn use_drop_zone() -> (Signal<DropZoneState>, impl FnMut(DropZoneState)) {
    let drop_state = use_signal(|| DropZoneState::Idle);
    
    let set_drop_state = {
        let mut drop_state = drop_state.clone();
        move |state: DropZoneState| {
            drop_state.set(state);
        }
    };
    
    (drop_state, set_drop_state)
}

/// Utility function to determine if a drop operation is valid
pub fn is_valid_drop_target(drag_files: &[FileEntry], target_path: &PathBuf) -> bool {
    // Check if we're not trying to drop files into themselves or their children
    for file in drag_files {
        if file.path == *target_path {
            return false; // Can't drop file onto itself
        }
        
        if file.is_directory && target_path.starts_with(&file.path) {
            return false; // Can't drop directory into its own child
        }
    }
    
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::SystemTime;

    fn create_test_file_entry(name: &str, is_directory: bool) -> FileEntry {
        FileEntry {
            path: PathBuf::from(name),
            name: name.to_string(),
            file_type: if is_directory {
                crate::services::file_system::FileType::Directory
            } else {
                crate::services::file_system::FileType::Text(
                    crate::services::file_system::TextFormat::Plain
                )
            },
            size: if is_directory { 0 } else { 100 },
            modified: SystemTime::now(),
            created: SystemTime::now(),
            is_directory,
            is_hidden: false,
            permissions: crate::services::file_system::FilePermissions::read_write(),
        }
    }

    #[test]
    fn test_drag_operation_from_modifiers() {
        assert_eq!(DragOperation::from_modifiers(false, false, false), DragOperation::Move);
        assert_eq!(DragOperation::from_modifiers(true, false, false), DragOperation::Copy);
        assert_eq!(DragOperation::from_modifiers(false, false, true), DragOperation::Link);
    }

    #[test]
    fn test_drag_state() {
        let mut state = DragState::default();
        assert!(!state.is_dragging);
        
        let files = vec![create_test_file_entry("test.txt", false)];
        state.start_drag(files, 100.0, 200.0, DragOperation::Copy);
        
        assert!(state.is_dragging);
        assert_eq!(state.drag_files.len(), 1);
        assert_eq!(state.operation, DragOperation::Copy);
        assert_eq!(state.start_position, (100.0, 200.0));
        
        state.update_position(150.0, 250.0);
        assert_eq!(state.current_position, (150.0, 250.0));
        assert!((state.get_drag_distance() - 70.71).abs() < 0.01); // sqrt(50^2 + 50^2)
        
        state.end_drag();
        assert!(!state.is_dragging);
        assert!(state.drag_files.is_empty());
    }

    #[test]
    fn test_drop_zone_state() {
        assert_eq!(DropZoneState::Idle.style_class(), "drop-zone-idle");
        assert_eq!(DropZoneState::DragOverValid.style_class(), "drop-zone-valid");
        
        assert!(DropZoneState::Idle.border_style().contains("transparent"));
        assert!(DropZoneState::DragOverValid.border_style().contains("#28a745"));
    }

    #[test]
    fn test_valid_drop_target() {
        let file = create_test_file_entry("test.txt", false);
        let folder = create_test_file_entry("folder", true);
        
        // Valid drops
        assert!(is_valid_drop_target(&[file.clone()], &PathBuf::from("other_folder")));
        assert!(is_valid_drop_target(&[folder.clone()], &PathBuf::from("other_folder")));
        
        // Invalid drops
        assert!(!is_valid_drop_target(&[file.clone()], &file.path)); // Same file
        assert!(!is_valid_drop_target(&[folder.clone()], &folder.path)); // Same folder
        
        // Test directory hierarchy
        let parent_folder = create_test_file_entry("parent", true);
        let child_path = PathBuf::from("parent/child");
        assert!(!is_valid_drop_target(&[parent_folder], &child_path)); // Can't drop parent into child
    }
}