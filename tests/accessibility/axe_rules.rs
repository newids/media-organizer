// Custom axe-core rules and configurations for MediaOrganizer
// Defines accessibility rules specific to file management and preview interfaces

use serde_json::{json, Value};

/// Get MediaOrganizer-specific axe-core rule configurations
pub fn get_media_organizer_rules() -> Value {
    json!({
        "rules": {
            // File tree accessibility
            "file-tree-structure": {
                "enabled": true,
                "options": {
                    "requiredParent": ["tree", "group"],
                    "requiredChild": ["treeitem"]
                }
            },
            
            // Preview panel accessibility
            "preview-panel-labels": {
                "enabled": true,
                "options": {
                    "requiredRoles": ["img", "region"],
                    "requiredLabels": ["aria-label", "aria-labelledby"]
                }
            },
            
            // Media controls accessibility
            "media-controls-keyboard": {
                "enabled": true,
                "options": {
                    "requiredControls": ["play", "pause", "volume", "seek"],
                    "keyboardAccessible": true
                }
            },
            
            // Status and live regions for file operations
            "file-operation-announcements": {
                "enabled": true,
                "options": {
                    "liveRegionTypes": ["polite", "assertive"],
                    "requiredForOperations": ["copy", "move", "delete", "rename"]
                }
            },
            
            // Activity bar and navigation
            "activity-bar-navigation": {
                "enabled": true,
                "options": {
                    "navigationRole": true,
                    "arrowKeyNavigation": true,
                    "focusManagement": true
                }
            }
        },
        
        "tags": {
            "media-organizer": [
                "file-tree-structure",
                "preview-panel-labels", 
                "media-controls-keyboard",
                "file-operation-announcements",
                "activity-bar-navigation"
            ]
        }
    })
}

/// WCAG 2.1 AA compliance rules specifically for file management interfaces
pub fn get_wcag_file_management_rules() -> Value {
    json!({
        "rules": {
            // 1.1.1 Non-text Content - File icons and previews
            "file-icon-alt-text": {
                "enabled": true,
                "impact": "critical",
                "wcag": "111"
            },
            
            // 1.3.1 Info and Relationships - File hierarchy
            "file-hierarchy-semantics": {
                "enabled": true,  
                "impact": "serious",
                "wcag": "131"
            },
            
            // 1.3.2 Meaningful Sequence - File list order
            "file-list-sequence": {
                "enabled": true,
                "impact": "serious", 
                "wcag": "132"
            },
            
            // 1.4.3 Contrast (Minimum) - File list readability  
            "file-list-contrast": {
                "enabled": true,
                "impact": "serious",
                "wcag": "143"
            },
            
            // 2.1.1 Keyboard - File operations
            "file-operations-keyboard": {
                "enabled": true,
                "impact": "critical",
                "wcag": "211"  
            },
            
            // 2.4.1 Bypass Blocks - Skip to file content
            "skip-to-files": {
                "enabled": true,
                "impact": "serious",
                "wcag": "241"
            },
            
            // 2.4.2 Page Titled - Current directory context
            "directory-context-title": {
                "enabled": true,
                "impact": "moderate",
                "wcag": "242"
            },
            
            // 2.4.3 Focus Order - File navigation flow
            "file-navigation-focus": {
                "enabled": true,
                "impact": "serious", 
                "wcag": "243"
            },
            
            // 2.4.6 Headings and Labels - File metadata
            "file-metadata-labels": {
                "enabled": true,
                "impact": "serious",
                "wcag": "246"
            },
            
            // 2.4.7 Focus Visible - File selection indicators
            "file-selection-focus": {
                "enabled": true,
                "impact": "serious",
                "wcag": "247"
            },
            
            // 3.2.1 On Focus - File preview stability  
            "file-preview-stable": {
                "enabled": true,
                "impact": "moderate",
                "wcag": "321"
            },
            
            // 3.2.2 On Input - Search/filter predictability
            "file-search-predictable": {
                "enabled": true,
                "impact": "moderate", 
                "wcag": "322"
            },
            
            // 4.1.2 Name, Role, Value - File controls
            "file-controls-accessible": {
                "enabled": true,
                "impact": "critical",
                "wcag": "412"
            }
        }
    })
}

/// High contrast and color accessibility rules for file management
pub fn get_visual_accessibility_rules() -> Value {
    json!({
        "rules": {
            // Color contrast for file type indicators
            "file-type-contrast": {
                "enabled": true,
                "impact": "serious",
                "options": {
                    "minContrastRatio": 4.5,
                    "largeTextRatio": 3.0
                }
            },
            
            // Focus indicators for file selection
            "file-focus-indicators": {
                "enabled": true, 
                "impact": "serious",
                "options": {
                    "minContrastRatio": 3.0,
                    "focusableElements": ["button", "[tabindex]", "a", "input"]
                }
            },
            
            // Selection state visibility
            "file-selection-visibility": {
                "enabled": true,
                "impact": "moderate",
                "options": {
                    "selectionIndicators": ["aria-selected", "aria-pressed", ":checked"]
                }
            },
            
            // Media preview accessibility
            "media-preview-accessible": {
                "enabled": true,
                "impact": "serious", 
                "options": {
                    "requiredControls": true,
                    "alternativeFormats": true
                }
            }
        }
    })
}

/// Keyboard navigation rules for file management interfaces
pub fn get_keyboard_navigation_rules() -> Value {
    json!({
        "rules": {
            // File tree keyboard navigation
            "file-tree-keyboard": {
                "enabled": true,
                "impact": "critical",
                "options": {
                    "arrowKeys": ["ArrowUp", "ArrowDown", "ArrowLeft", "ArrowRight"],
                    "actionKeys": ["Enter", "Space"],
                    "navigationKeys": ["Home", "End", "PageUp", "PageDown"]
                }
            },
            
            // Preview panel keyboard controls
            "preview-keyboard-controls": {
                "enabled": true,
                "impact": "serious",
                "options": {
                    "zoomControls": ["+", "-", "0"],
                    "panControls": ["ArrowKeys"],
                    "toggleControls": ["Space", "Enter"]
                }
            },
            
            // Tab order in file operations
            "file-operations-tab-order": {
                "enabled": true,
                "impact": "serious", 
                "options": {
                    "logicalOrder": true,
                    "noSkippedElements": true,
                    "visibleFocusIndicators": true
                }
            },
            
            // Global keyboard shortcuts
            "global-shortcuts-accessible": {
                "enabled": true,
                "impact": "moderate",
                "options": {
                    "standardShortcuts": true,
                    "documentedShortcuts": true,
                    "conflictFree": true
                }
            }
        }
    })
}

/// Screen reader and assistive technology rules
pub fn get_screen_reader_rules() -> Value {
    json!({
        "rules": {
            // File announcements for screen readers
            "file-announcements": {
                "enabled": true,
                "impact": "critical",
                "options": {
                    "liveRegions": true,
                    "statusUpdates": true,
                    "operationFeedback": true
                }
            },
            
            // Semantic structure for file content
            "file-semantic-structure": {
                "enabled": true,
                "impact": "serious",
                "options": {
                    "landmarkRoles": ["main", "navigation", "complementary", "region"],
                    "headingHierarchy": true,
                    "listStructure": true
                }
            },
            
            // ARIA labels for complex file operations
            "file-aria-labels": {
                "enabled": true,
                "impact": "serious",
                "options": {
                    "descriptiveLabels": true,
                    "contextualInformation": true,
                    "operationStatus": true
                }
            },
            
            // Progressive disclosure for file metadata
            "file-metadata-disclosure": {
                "enabled": true,
                "impact": "moderate", 
                "options": {
                    "expandableContent": true,
                    "ariaExpanded": true,
                    "ariaControls": true
                }
            }
        }
    })
}

/// Combined rule configuration for comprehensive testing
pub fn get_comprehensive_rules() -> Value {
    let media_organizer_rules = get_media_organizer_rules();
    let wcag_rules = get_wcag_file_management_rules();
    let visual_rules = get_visual_accessibility_rules();
    let keyboard_rules = get_keyboard_navigation_rules();
    let screen_reader_rules = get_screen_reader_rules();
    
    // Merge all rule sets
    json!({
        "rules": {
            // Merge all rule objects
            ..media_organizer_rules["rules"].as_object().unwrap().clone(),
            ..wcag_rules["rules"].as_object().unwrap().clone(),
            ..visual_rules["rules"].as_object().unwrap().clone(),
            ..keyboard_rules["rules"].as_object().unwrap().clone(),
            ..screen_reader_rules["rules"].as_object().unwrap().clone()
        },
        "tags": ["wcag2a", "wcag2aa", "wcag21aa", "best-practice", "media-organizer"],
        "resultTypes": ["violations", "passes", "incomplete", "inapplicable"],
        "reporter": "v2",
        "runOnly": {
            "type": "tag",
            "values": ["wcag2a", "wcag2aa", "wcag21aa"]
        },
        "environment": {
            "orientationAngle": 0,
            "windowWidth": 1920,
            "windowHeight": 1080
        }
    })
}