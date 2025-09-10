use super::color_theme::{ThemeColors, TokenColor, TokenSettings, SemanticColors, SemanticColorRule};
use std::collections::HashMap;

impl ThemeColors {
    /// VSCode Dark+ theme colors
    pub fn dark_plus() -> Self {
        Self {
            // Base colors
            foreground: "#CCCCCC".to_string(),
            foreground_secondary: "#9D9D9D".to_string(),
            error_foreground: "#F48771".to_string(),
            warning_foreground: "#CCA700".to_string(),
            info_foreground: "#75BEFF".to_string(),
            success_foreground: "#89D185".to_string(),
            
            // Background colors
            background: "#1E1E1E".to_string(),
            secondary_background: "#252526".to_string(),
            tertiary_background: "#2D2D30".to_string(),
            
            // Border and separator colors
            border: "#3E3E42".to_string(),
            border_secondary: "#2D2D30".to_string(),
            focus_border: "#007ACC".to_string(),
            drop_shadow: "#00000080".to_string(),
            
            // Button colors
            button_background: "#0E639C".to_string(),
            button_foreground: "#FFFFFF".to_string(),
            button_hover_background: "#1177BB".to_string(),
            button_disabled_background: "#2D2D30".to_string(),
            button_disabled_foreground: "#CCCCCC80".to_string(),
            
            // Input colors
            input_background: "#3C3C3C".to_string(),
            input_foreground: "#CCCCCC".to_string(),
            input_border: "#3E3E42".to_string(),
            input_placeholder_foreground: "#A6A6A6".to_string(),
            input_validation_info_background: "#063B49".to_string(),
            input_validation_info_foreground: "#75BEFF".to_string(),
            input_validation_warning_background: "#352A05".to_string(),
            input_validation_warning_foreground: "#CCA700".to_string(),
            input_validation_error_background: "#5A1D1D".to_string(),
            input_validation_error_foreground: "#F48771".to_string(),
            
            // Dropdown colors
            dropdown_background: "#3C3C3C".to_string(),
            dropdown_foreground: "#CCCCCC".to_string(),
            dropdown_border: "#3E3E42".to_string(),
            dropdown_list_background: "#383838".to_string(),
            
            // List colors
            list_background: "#252526".to_string(),
            list_foreground: "#CCCCCC".to_string(),
            list_active_selection_background: "#094771".to_string(),
            list_active_selection_foreground: "#FFFFFF".to_string(),
            list_inactive_selection_background: "#37373D".to_string(),
            list_inactive_selection_foreground: "#CCCCCC".to_string(),
            list_hover_background: "#2A2D2E".to_string(),
            list_hover_foreground: "#CCCCCC".to_string(),
            list_focus_background: "#062F4A".to_string(),
            list_focus_foreground: "#CCCCCC".to_string(),
            list_focus_outline: "#007ACC".to_string(),
            list_drop_background: "#062F4A".to_string(),
            list_highlight_foreground: "#75BEFF".to_string(),
            list_invalid_item_foreground: "#B89500".to_string(),
            list_error_foreground: "#F88070".to_string(),
            list_warning_foreground: "#CCA700".to_string(),
            list_filter_widget_background: "#252526".to_string(),
            list_filter_widget_outline: "#00000000".to_string(),
            list_filter_widget_no_matches_outline: "#BE1100".to_string(),
            
            // Activity Bar colors
            activity_bar_background: "#333333".to_string(),
            activity_bar_foreground: "#FFFFFF".to_string(),
            activity_bar_inactive_foreground: "#FFFFFF66".to_string(),
            activity_bar_border: "#00000000".to_string(),
            activity_bar_drag_and_drop_border: "#FFFFFF".to_string(),
            activity_bar_active_background: "#00000000".to_string(),
            activity_bar_active_foreground: "#FFFFFF".to_string(),
            activity_bar_active_focus_border: "#007ACC".to_string(),
            activity_bar_badge_background: "#007ACC".to_string(),
            activity_bar_badge_foreground: "#FFFFFF".to_string(),
            
            // Side Bar colors
            side_bar_background: "#252526".to_string(),
            side_bar_foreground: "#CCCCCC".to_string(),
            side_bar_border: "#2D2D30".to_string(),
            side_bar_title_foreground: "#CCCCCC".to_string(),
            side_bar_drag_and_drop_background: "#383B3D".to_string(),
            side_bar_section_header_background: "#00000000".to_string(),
            side_bar_section_header_foreground: "#CCCCCC".to_string(),
            side_bar_section_header_border: "#CCCCCC33".to_string(),
            
            // Minimap colors
            minimap_background: "#1E1E1E".to_string(),
            minimap_selection_highlight: "#264F78".to_string(),
            minimap_find_match_highlight: "#515C6A".to_string(),
            minimap_selection_occurrence_highlight: "#676767".to_string(),
            
            // Editor Group & Tab colors
            editor_group_border: "#2D2D30".to_string(),
            editor_group_drag_and_drop_background: "#25252680".to_string(),
            editor_group_empty_background: "#1E1E1E".to_string(),
            editor_group_focused_empty_border: "#007ACC".to_string(),
            editor_group_header_tabs_background: "#2D2D30".to_string(),
            editor_group_header_tabs_border: "#00000000".to_string(),
            editor_group_header_no_tabs_background: "#1E1E1E".to_string(),
            
            // Tab colors
            tab_active_background: "#1E1E1E".to_string(),
            tab_active_foreground: "#FFFFFF".to_string(),
            tab_active_border: "#00000000".to_string(),
            tab_active_border_top: "#007ACC".to_string(),
            tab_inactive_background: "#2D2D30".to_string(),
            tab_inactive_foreground: "#FFFFFF80".to_string(),
            tab_inactive_border: "#00000000".to_string(),
            tab_inactive_border_top: "#00000000".to_string(),
            tab_hover_background: "#1E1E1E".to_string(),
            tab_hover_foreground: "#FFFFFF".to_string(),
            tab_hover_border: "#00000000".to_string(),
            tab_unfocused_active_background: "#1E1E1E".to_string(),
            tab_unfocused_active_foreground: "#FFFFFF80".to_string(),
            tab_unfocused_active_border: "#00000000".to_string(),
            tab_unfocused_active_border_top: "#007ACC80".to_string(),
            tab_unfocused_inactive_background: "#2D2D30".to_string(),
            tab_unfocused_inactive_foreground: "#FFFFFF40".to_string(),
            tab_unfocused_inactive_border: "#00000000".to_string(),
            tab_unfocused_inactive_border_top: "#00000000".to_string(),
            tab_unfocused_hover_background: "#1E1E1E".to_string(),
            tab_unfocused_hover_foreground: "#FFFFFF80".to_string(),
            tab_unfocused_hover_border: "#00000000".to_string(),
            tab_last_pinned_border: "#CCCCCC33".to_string(),
            
            // Editor colors
            editor_background: "#1E1E1E".to_string(),
            editor_foreground: "#D4D4D4".to_string(),
            editor_line_highlight_background: "#FFFFFF0A".to_string(),
            editor_line_highlight_border: "#00000000".to_string(),
            editor_range_highlight_background: "#FFFFFF0D".to_string(),
            editor_range_highlight_border: "#00000000".to_string(),
            editor_cursor_background: "#000000".to_string(),
            editor_cursor_foreground: "#AEAFAD".to_string(),
            editor_white_space_foreground: "#3E3E42".to_string(),
            editor_indent_guide_background: "#404040".to_string(),
            editor_indent_guide_active_background: "#707070".to_string(),
            editor_line_number_foreground: "#858585".to_string(),
            editor_line_number_active_foreground: "#C6C6C6".to_string(),
            editor_ruler_foreground: "#5A5A5A".to_string(),
            editor_code_lens_foreground: "#999999".to_string(),
            editor_bracket_match_background: "#0064001A".to_string(),
            editor_bracket_match_border: "#888888".to_string(),
            editor_overview_ruler_border: "#7F7F7F4D".to_string(),
            editor_overview_ruler_find_match_foreground: "#D186167E".to_string(),
            editor_overview_ruler_range_highlight_foreground: "#007ACC99".to_string(),
            editor_overview_ruler_selection_highlight_foreground: "#A0A0A0CC".to_string(),
            editor_overview_ruler_word_highlight_foreground: "#A0A0A0CC".to_string(),
            editor_overview_ruler_word_highlight_strong_foreground: "#C0A0C0CC".to_string(),
            editor_overview_ruler_modified_foreground: "#007ACC".to_string(),
            editor_overview_ruler_added_foreground: "#007ACC".to_string(),
            editor_overview_ruler_deleted_foreground: "#007ACC".to_string(),
            editor_overview_ruler_error_foreground: "#FF1212B3".to_string(),
            editor_overview_ruler_warning_foreground: "#CCA700".to_string(),
            editor_overview_ruler_info_foreground: "#007ACC".to_string(),
            editor_overview_ruler_bracket_match_foreground: "#A0A0A0".to_string(),
            
            // Find Widget colors
            editor_find_match_background: "#515C6A".to_string(),
            editor_find_match_highlight_background: "#EA5C0055".to_string(),
            editor_find_range_highlight_background: "#3A3D4166".to_string(),
            editor_find_match_border: "#74879F".to_string(),
            editor_find_range_highlight_border: "#00000000".to_string(),
            editor_hovered_highlight_background: "#264F7840".to_string(),
            editor_hovered_word_highlight_background: "#264F7840".to_string(),
            editor_hovered_word_highlight_border: "#264F78AA".to_string(),
            editor_link_active_foreground: "#4E94CE".to_string(),
            
            // Selection colors
            editor_selection_background: "#264F78".to_string(),
            editor_selection_foreground: "#00000000".to_string(),
            editor_inactive_selection_background: "#3A3D41".to_string(),
            editor_selection_highlight_background: "#ADD6FF26".to_string(),
            editor_selection_highlight_border: "#00000000".to_string(),
            
            // Word highlight colors
            editor_word_highlight_background: "#575757B8".to_string(),
            editor_word_highlight_border: "#00000000".to_string(),
            editor_word_highlight_strong_background: "#004972B8".to_string(),
            editor_word_highlight_strong_border: "#00000000".to_string(),
            
            // Panel colors
            panel_background: "#1E1E1E".to_string(),
            panel_border: "#3E3E42".to_string(),
            panel_drag_and_drop_border: "#FFFFFF".to_string(),
            panel_section_border: "#3E3E42".to_string(),
            panel_section_drag_and_drop_background: "#383B3D".to_string(),
            panel_section_header_background: "#00000080".to_string(),
            panel_section_header_foreground: "#CCCCCC".to_string(),
            panel_section_header_border: "#CCCCCC33".to_string(),
            
            // Status Bar colors
            status_bar_background: "#007ACC".to_string(),
            status_bar_foreground: "#FFFFFF".to_string(),
            status_bar_border: "#00000000".to_string(),
            status_bar_debugging_background: "#CC6633".to_string(),
            status_bar_debugging_foreground: "#FFFFFF".to_string(),
            status_bar_debugging_border: "#00000000".to_string(),
            status_bar_no_folder_background: "#68217A".to_string(),
            status_bar_no_folder_foreground: "#FFFFFF".to_string(),
            status_bar_no_folder_border: "#00000000".to_string(),
            status_bar_item_active_background: "#FFFFFF25".to_string(),
            status_bar_item_hover_background: "#FFFFFF1F".to_string(),
            status_bar_item_prominent_background: "#00000080".to_string(),
            status_bar_item_prominent_hover_background: "#0000004D".to_string(),
            status_bar_item_remote_background: "#16825D".to_string(),
            status_bar_item_remote_foreground: "#FFFFFF".to_string(),
            
            // Title Bar colors
            title_bar_active_background: "#3C3C3C".to_string(),
            title_bar_active_foreground: "#CCCCCC".to_string(),
            title_bar_inactive_background: "#3C3C3C".to_string(),
            title_bar_inactive_foreground: "#6F6F6F".to_string(),
            title_bar_border: "#60606060".to_string(),
            
            // Menu colors
            menu_foreground: "#CCCCCC".to_string(),
            menu_background: "#383838".to_string(),
            menu_selection_foreground: "#FFFFFF".to_string(),
            menu_selection_background: "#094771".to_string(),
            menu_selection_border: "#00000000".to_string(),
            menu_separator_background: "#BBBBBB".to_string(),
            menu_border: "#454545".to_string(),
            
            // Notification colors
            notifications_center_border: "#3E3E42".to_string(),
            notifications_toast_border: "#3E3E42".to_string(),
            notifications_foreground: "#CCCCCC".to_string(),
            notifications_background: "#252526".to_string(),
            notifications_border: "#3E3E42".to_string(),
            notification_center_header_foreground: "#CCCCCC".to_string(),
            notification_center_header_background: "#2D2D30".to_string(),
            notification_toast_background: "#252526".to_string(),
            notifications_links: "#3794FF".to_string(),
            notifications_center_header_border: "#3E3E42".to_string(),
            
            // Extensions colors
            extension_button_prominent_foreground: "#FFFFFF".to_string(),
            extension_button_prominent_background: "#327E36".to_string(),
            extension_button_prominent_hover_background: "#28632F".to_string(),
            
            // Picklist colors
            picklist_group_foreground: "#3794FF".to_string(),
            picklist_group_border: "#3E3E42".to_string(),
            
            // Terminal colors
            terminal_background: "#1E1E1E".to_string(),
            terminal_foreground: "#CCCCCC".to_string(),
            terminal_cursor_background: "#FFFFFF".to_string(),
            terminal_cursor_foreground: "#000000".to_string(),
            terminal_selection_background: "#FFFFFF40".to_string(),
            terminal_inactive_selection_background: "#FFFFFF20".to_string(),
            
            // Git decoration colors
            git_decoration_modified_resource_foreground: "#E2C08D".to_string(),
            git_decoration_deleted_resource_foreground: "#C74E39".to_string(),
            git_decoration_untracked_resource_foreground: "#73C991".to_string(),
            git_decoration_ignored_resource_foreground: "#8C8C8C".to_string(),
            git_decoration_conflicting_resource_foreground: "#E4676B".to_string(),
            git_decoration_staged_modified_resource_foreground: "#E2C08D".to_string(),
            git_decoration_staged_deleted_resource_foreground: "#C74E39".to_string(),
            
            // Settings colors
            settings_header_foreground: "#E7E7E7".to_string(),
            settings_modified_item_indicator: "#007ACC".to_string(),
            settings_dropdown_background: "#3C3C3C".to_string(),
            settings_dropdown_foreground: "#CCCCCC".to_string(),
            settings_dropdown_border: "#3E3E42".to_string(),
            settings_dropdown_list_border: "#454545".to_string(),
            settings_checkbox_background: "#3C3C3C".to_string(),
            settings_checkbox_foreground: "#F0F0F0".to_string(),
            settings_checkbox_border: "#3E3E42".to_string(),
            settings_row_hover_background: "#2A2D2E".to_string(),
            settings_text_input_background: "#3C3C3C".to_string(),
            settings_text_input_foreground: "#CCCCCC".to_string(),
            settings_text_input_border: "#3E3E42".to_string(),
            settings_number_input_background: "#3C3C3C".to_string(),
            settings_number_input_foreground: "#CCCCCC".to_string(),
            settings_number_input_border: "#3E3E42".to_string(),
            
            // Progress bar colors
            progress_bar_background: "#0E70C0".to_string(),
            
            // Breadcrumb colors
            breadcrumb_foreground: "#CCCCCCCC".to_string(),
            breadcrumb_background: "#1E1E1E".to_string(),
            breadcrumb_focus_foreground: "#E0E0E0".to_string(),
            breadcrumb_active_selection_foreground: "#007ACC".to_string(),
            breadcrumb_picker_background: "#252526".to_string(),
            
            // Scrollbar colors
            scrollbar_shadow: "#000000".to_string(),
            scrollbar_slider_background: "#79797966".to_string(),
            scrollbar_slider_hover_background: "#646464B3".to_string(),
            scrollbar_slider_active_background: "#BFBFBF66".to_string(),
            
            // Badge colors
            badge_background: "#007ACC".to_string(),
            badge_foreground: "#FFFFFF".to_string(),
        }
    }
    
    /// VSCode Light+ theme colors
    pub fn light_plus() -> Self {
        Self {
            // Base colors
            foreground: "#616161".to_string(),
            foreground_secondary: "#8E8E8E".to_string(),
            error_foreground: "#A1260D".to_string(),
            warning_foreground: "#BF8803".to_string(),
            info_foreground: "#1976D2".to_string(),
            success_foreground: "#14CE14".to_string(),
            
            // Background colors
            background: "#FFFFFF".to_string(),
            secondary_background: "#F3F3F3".to_string(),
            tertiary_background: "#ECECEC".to_string(),
            
            // Border and separator colors
            border: "#C8C8C8".to_string(),
            border_secondary: "#E5E5E5".to_string(),
            focus_border: "#0090F1".to_string(),
            drop_shadow: "#00000026".to_string(),
            
            // Button colors
            button_background: "#007ACC".to_string(),
            button_foreground: "#FFFFFF".to_string(),
            button_hover_background: "#106BA3".to_string(),
            button_disabled_background: "#F3F3F3".to_string(),
            button_disabled_foreground: "#8E8E8E".to_string(),
            
            // Input colors
            input_background: "#FFFFFF".to_string(),
            input_foreground: "#000000".to_string(),
            input_border: "#CECECE".to_string(),
            input_placeholder_foreground: "#767676".to_string(),
            input_validation_info_background: "#D6ECF2".to_string(),
            input_validation_info_foreground: "#1976D2".to_string(),
            input_validation_warning_background: "#F6F5D2".to_string(),
            input_validation_warning_foreground: "#BF8803".to_string(),
            input_validation_error_background: "#F2DEDE".to_string(),
            input_validation_error_foreground: "#A1260D".to_string(),
            
            // Dropdown colors
            dropdown_background: "#FFFFFF".to_string(),
            dropdown_foreground: "#000000".to_string(),
            dropdown_border: "#CECECE".to_string(),
            dropdown_list_background: "#F3F3F3".to_string(),
            
            // List colors
            list_background: "#FFFFFF".to_string(),
            list_foreground: "#000000".to_string(),
            list_active_selection_background: "#0060C0".to_string(),
            list_active_selection_foreground: "#FFFFFF".to_string(),
            list_inactive_selection_background: "#E4E6F1".to_string(),
            list_inactive_selection_foreground: "#000000".to_string(),
            list_hover_background: "#F0F0F0".to_string(),
            list_hover_foreground: "#000000".to_string(),
            list_focus_background: "#D6EBFF".to_string(),
            list_focus_foreground: "#000000".to_string(),
            list_focus_outline: "#0090F1".to_string(),
            list_drop_background: "#D6EBFF".to_string(),
            list_highlight_foreground: "#0066BF".to_string(),
            list_invalid_item_foreground: "#B89500".to_string(),
            list_error_foreground: "#A1260D".to_string(),
            list_warning_foreground: "#BF8803".to_string(),
            list_filter_widget_background: "#FFFFFF".to_string(),
            list_filter_widget_outline: "#00000000".to_string(),
            list_filter_widget_no_matches_outline: "#BE1100".to_string(),
            
            // Activity Bar colors
            activity_bar_background: "#2C2C2C".to_string(),
            activity_bar_foreground: "#FFFFFF".to_string(),
            activity_bar_inactive_foreground: "#FFFFFF66".to_string(),
            activity_bar_border: "#00000000".to_string(),
            activity_bar_drag_and_drop_border: "#FFFFFF".to_string(),
            activity_bar_active_background: "#00000000".to_string(),
            activity_bar_active_foreground: "#FFFFFF".to_string(),
            activity_bar_active_focus_border: "#007ACC".to_string(),
            activity_bar_badge_background: "#007ACC".to_string(),
            activity_bar_badge_foreground: "#FFFFFF".to_string(),
            
            // Side Bar colors
            side_bar_background: "#F3F3F3".to_string(),
            side_bar_foreground: "#000000".to_string(),
            side_bar_border: "#E5E5E5".to_string(),
            side_bar_title_foreground: "#6F6F6F".to_string(),
            side_bar_drag_and_drop_background: "#E1E1E1".to_string(),
            side_bar_section_header_background: "#00000000".to_string(),
            side_bar_section_header_foreground: "#6F6F6F".to_string(),
            side_bar_section_header_border: "#61616133".to_string(),
            
            // Minimap colors
            minimap_background: "#FFFFFF".to_string(),
            minimap_selection_highlight: "#ADD6FF".to_string(),
            minimap_find_match_highlight: "#BDE4FF".to_string(),
            minimap_selection_occurrence_highlight: "#C9C9C9".to_string(),
            
            // Editor Group & Tab colors
            editor_group_border: "#E7E7E7".to_string(),
            editor_group_drag_and_drop_background: "#F3F3F380".to_string(),
            editor_group_empty_background: "#FFFFFF".to_string(),
            editor_group_focused_empty_border: "#0090F1".to_string(),
            editor_group_header_tabs_background: "#F3F3F3".to_string(),
            editor_group_header_tabs_border: "#00000000".to_string(),
            editor_group_header_no_tabs_background: "#FFFFFF".to_string(),
            
            // Tab colors
            tab_active_background: "#FFFFFF".to_string(),
            tab_active_foreground: "#333333".to_string(),
            tab_active_border: "#00000000".to_string(),
            tab_active_border_top: "#007ACC".to_string(),
            tab_inactive_background: "#ECECEC".to_string(),
            tab_inactive_foreground: "#33333380".to_string(),
            tab_inactive_border: "#00000000".to_string(),
            tab_inactive_border_top: "#00000000".to_string(),
            tab_hover_background: "#FFFFFF".to_string(),
            tab_hover_foreground: "#333333".to_string(),
            tab_hover_border: "#00000000".to_string(),
            tab_unfocused_active_background: "#FFFFFF".to_string(),
            tab_unfocused_active_foreground: "#33333380".to_string(),
            tab_unfocused_active_border: "#00000000".to_string(),
            tab_unfocused_active_border_top: "#007ACC80".to_string(),
            tab_unfocused_inactive_background: "#ECECEC".to_string(),
            tab_unfocused_inactive_foreground: "#33333340".to_string(),
            tab_unfocused_inactive_border: "#00000000".to_string(),
            tab_unfocused_inactive_border_top: "#00000000".to_string(),
            tab_unfocused_hover_background: "#FFFFFF".to_string(),
            tab_unfocused_hover_foreground: "#33333380".to_string(),
            tab_unfocused_hover_border: "#00000000".to_string(),
            tab_last_pinned_border: "#61616133".to_string(),
            
            // Editor colors
            editor_background: "#FFFFFF".to_string(),
            editor_foreground: "#000000".to_string(),
            editor_line_highlight_background: "#0000000A".to_string(),
            editor_line_highlight_border: "#EEEEEE".to_string(),
            editor_range_highlight_background: "#FDFF0033".to_string(),
            editor_range_highlight_border: "#00000000".to_string(),
            editor_cursor_background: "#FFFFFF".to_string(),
            editor_cursor_foreground: "#000000".to_string(),
            editor_white_space_foreground: "#33333333".to_string(),
            editor_indent_guide_background: "#D3D3D3".to_string(),
            editor_indent_guide_active_background: "#939393".to_string(),
            editor_line_number_foreground: "#237893".to_string(),
            editor_line_number_active_foreground: "#0B216F".to_string(),
            editor_ruler_foreground: "#D3D3D3".to_string(),
            editor_code_lens_foreground: "#919191".to_string(),
            editor_bracket_match_background: "#0064001A".to_string(),
            editor_bracket_match_border: "#B9B9B9".to_string(),
            editor_overview_ruler_border: "#7F7F7F4D".to_string(),
            editor_overview_ruler_find_match_foreground: "#A8AC94".to_string(),
            editor_overview_ruler_range_highlight_foreground: "#007ACC4D".to_string(),
            editor_overview_ruler_selection_highlight_foreground: "#A0A0A04D".to_string(),
            editor_overview_ruler_word_highlight_foreground: "#A0A0A04D".to_string(),
            editor_overview_ruler_word_highlight_strong_foreground: "#C0A0C04D".to_string(),
            editor_overview_ruler_modified_foreground: "#007ACC".to_string(),
            editor_overview_ruler_added_foreground: "#007ACC".to_string(),
            editor_overview_ruler_deleted_foreground: "#007ACC".to_string(),
            editor_overview_ruler_error_foreground: "#FF1212B3".to_string(),
            editor_overview_ruler_warning_foreground: "#BF8803".to_string(),
            editor_overview_ruler_info_foreground: "#007ACC".to_string(),
            editor_overview_ruler_bracket_match_foreground: "#A0A0A0".to_string(),
            
            // Find Widget colors
            editor_find_match_background: "#A8AC94".to_string(),
            editor_find_match_highlight_background: "#EA5C0055".to_string(),
            editor_find_range_highlight_background: "#B4B4B466".to_string(),
            editor_find_match_border: "#457B9D".to_string(),
            editor_find_range_highlight_border: "#00000000".to_string(),
            editor_hovered_highlight_background: "#ADD6FF26".to_string(),
            editor_hovered_word_highlight_background: "#ADD6FF26".to_string(),
            editor_hovered_word_highlight_border: "#ADD6FF80".to_string(),
            editor_link_active_foreground: "#0000FF".to_string(),
            
            // Selection colors
            editor_selection_background: "#ADD6FF".to_string(),
            editor_selection_foreground: "#00000000".to_string(),
            editor_inactive_selection_background: "#E5EBF1".to_string(),
            editor_selection_highlight_background: "#ADD6FF66".to_string(),
            editor_selection_highlight_border: "#00000000".to_string(),
            
            // Word highlight colors
            editor_word_highlight_background: "#57575740".to_string(),
            editor_word_highlight_border: "#00000000".to_string(),
            editor_word_highlight_strong_background: "#004972A1".to_string(),
            editor_word_highlight_strong_border: "#00000000".to_string(),
            
            // Panel colors
            panel_background: "#FFFFFF".to_string(),
            panel_border: "#80808059".to_string(),
            panel_drag_and_drop_border: "#000000".to_string(),
            panel_section_border: "#80808059".to_string(),
            panel_section_drag_and_drop_background: "#E1E1E1".to_string(),
            panel_section_header_background: "#80808033".to_string(),
            panel_section_header_foreground: "#6F6F6F".to_string(),
            panel_section_header_border: "#61616180".to_string(),
            
            // Status Bar colors
            status_bar_background: "#007ACC".to_string(),
            status_bar_foreground: "#FFFFFF".to_string(),
            status_bar_border: "#007ACC".to_string(),
            status_bar_debugging_background: "#CC6633".to_string(),
            status_bar_debugging_foreground: "#FFFFFF".to_string(),
            status_bar_debugging_border: "#CC6633".to_string(),
            status_bar_no_folder_background: "#68217A".to_string(),
            status_bar_no_folder_foreground: "#FFFFFF".to_string(),
            status_bar_no_folder_border: "#68217A".to_string(),
            status_bar_item_active_background: "#FFFFFF25".to_string(),
            status_bar_item_hover_background: "#FFFFFF1F".to_string(),
            status_bar_item_prominent_background: "#00000080".to_string(),
            status_bar_item_prominent_hover_background: "#0000004D".to_string(),
            status_bar_item_remote_background: "#16825D".to_string(),
            status_bar_item_remote_foreground: "#FFFFFF".to_string(),
            
            // Title Bar colors
            title_bar_active_background: "#DDDDDD".to_string(),
            title_bar_active_foreground: "#333333".to_string(),
            title_bar_inactive_background: "#DDDDDD".to_string(),
            title_bar_inactive_foreground: "#6F6F6F".to_string(),
            title_bar_border: "#60606060".to_string(),
            
            // Menu colors
            menu_foreground: "#616161".to_string(),
            menu_background: "#F6F6F6".to_string(),
            menu_selection_foreground: "#FFFFFF".to_string(),
            menu_selection_background: "#0060C0".to_string(),
            menu_selection_border: "#00000000".to_string(),
            menu_separator_background: "#888888".to_string(),
            menu_border: "#CCCCCC".to_string(),
            
            // Notification colors
            notifications_center_border: "#C8C8C8".to_string(),
            notifications_toast_border: "#C8C8C8".to_string(),
            notifications_foreground: "#616161".to_string(),
            notifications_background: "#F3F3F3".to_string(),
            notifications_border: "#C8C8C8".to_string(),
            notification_center_header_foreground: "#616161".to_string(),
            notification_center_header_background: "#ECECEC".to_string(),
            notification_toast_background: "#F3F3F3".to_string(),
            notifications_links: "#006AB1".to_string(),
            notifications_center_header_border: "#C8C8C8".to_string(),
            
            // Extensions colors
            extension_button_prominent_foreground: "#FFFFFF".to_string(),
            extension_button_prominent_background: "#327E36".to_string(),
            extension_button_prominent_hover_background: "#2D6A31".to_string(),
            
            // Picklist colors
            picklist_group_foreground: "#006AB1".to_string(),
            picklist_group_border: "#C8C8C8".to_string(),
            
            // Terminal colors
            terminal_background: "#FFFFFF".to_string(),
            terminal_foreground: "#000000".to_string(),
            terminal_cursor_background: "#000000".to_string(),
            terminal_cursor_foreground: "#FFFFFF".to_string(),
            terminal_selection_background: "#0000FF40".to_string(),
            terminal_inactive_selection_background: "#0000FF20".to_string(),
            
            // Git decoration colors
            git_decoration_modified_resource_foreground: "#E2C08D".to_string(),
            git_decoration_deleted_resource_foreground: "#C74E39".to_string(),
            git_decoration_untracked_resource_foreground: "#73C991".to_string(),
            git_decoration_ignored_resource_foreground: "#8C8C8C".to_string(),
            git_decoration_conflicting_resource_foreground: "#E4676B".to_string(),
            git_decoration_staged_modified_resource_foreground: "#E2C08D".to_string(),
            git_decoration_staged_deleted_resource_foreground: "#C74E39".to_string(),
            
            // Settings colors
            settings_header_foreground: "#444444".to_string(),
            settings_modified_item_indicator: "#007ACC".to_string(),
            settings_dropdown_background: "#FFFFFF".to_string(),
            settings_dropdown_foreground: "#000000".to_string(),
            settings_dropdown_border: "#CECECE".to_string(),
            settings_dropdown_list_border: "#C8C8C8".to_string(),
            settings_checkbox_background: "#FFFFFF".to_string(),
            settings_checkbox_foreground: "#000000".to_string(),
            settings_checkbox_border: "#CECECE".to_string(),
            settings_row_hover_background: "#F0F0F0".to_string(),
            settings_text_input_background: "#FFFFFF".to_string(),
            settings_text_input_foreground: "#000000".to_string(),
            settings_text_input_border: "#CECECE".to_string(),
            settings_number_input_background: "#FFFFFF".to_string(),
            settings_number_input_foreground: "#000000".to_string(),
            settings_number_input_border: "#CECECE".to_string(),
            
            // Progress bar colors
            progress_bar_background: "#0E70C0".to_string(),
            
            // Breadcrumb colors
            breadcrumb_foreground: "#616161CC".to_string(),
            breadcrumb_background: "#FFFFFF".to_string(),
            breadcrumb_focus_foreground: "#000000".to_string(),
            breadcrumb_active_selection_foreground: "#007ACC".to_string(),
            breadcrumb_picker_background: "#F3F3F3".to_string(),
            
            // Scrollbar colors
            scrollbar_shadow: "#DDDDDD".to_string(),
            scrollbar_slider_background: "#79797966".to_string(),
            scrollbar_slider_hover_background: "#646464B3".to_string(),
            scrollbar_slider_active_background: "#00000099".to_string(),
            
            // Badge colors
            badge_background: "#007ACC".to_string(),
            badge_foreground: "#FFFFFF".to_string(),
        }
    }
    
    /// High contrast dark theme colors
    pub fn high_contrast_dark() -> Self {
        let mut base = Self::dark_plus();
        
        // Override with high contrast values
        base.foreground = "#FFFFFF".to_string();
        base.background = "#000000".to_string();
        base.border = "#6FC3DF".to_string();
        base.focus_border = "#F38518".to_string();
        base.list_active_selection_background = "#FFFFFF".to_string();
        base.list_active_selection_foreground = "#000000".to_string();
        base.editor_cursor_foreground = "#FFFFFF".to_string();
        base.editor_selection_background = "#FFFFFF".to_string();
        base.editor_selection_foreground = "#000000".to_string();
        
        base
    }
    
    /// High contrast light theme colors
    pub fn high_contrast_light() -> Self {
        let mut base = Self::light_plus();
        
        // Override with high contrast values
        base.foreground = "#000000".to_string();
        base.background = "#FFFFFF".to_string();
        base.border = "#0F4A85".to_string();
        base.focus_border = "#BF0000".to_string();
        base.list_active_selection_background = "#000000".to_string();
        base.list_active_selection_foreground = "#FFFFFF".to_string();
        base.editor_cursor_foreground = "#000000".to_string();
        base.editor_selection_background = "#000000".to_string();
        base.editor_selection_foreground = "#FFFFFF".to_string();
        
        base
    }
}

impl TokenColor {
    /// Dark+ token colors for syntax highlighting
    pub fn dark_plus_tokens() -> Vec<Self> {
        vec![
            TokenColor {
                name: Some("Comment".to_string()),
                scope: vec!["comment".to_string()],
                settings: TokenSettings {
                    foreground: Some("#6A9955".to_string()),
                    background: None,
                    font_style: Some("italic".to_string()),
                },
            },
            TokenColor {
                name: Some("Keyword".to_string()),
                scope: vec!["keyword".to_string(), "storage.type".to_string(), "storage.modifier".to_string()],
                settings: TokenSettings {
                    foreground: Some("#569CD6".to_string()),
                    background: None,
                    font_style: None,
                },
            },
            TokenColor {
                name: Some("String".to_string()),
                scope: vec!["string".to_string()],
                settings: TokenSettings {
                    foreground: Some("#CE9178".to_string()),
                    background: None,
                    font_style: None,
                },
            },
            TokenColor {
                name: Some("Number".to_string()),
                scope: vec!["constant.numeric".to_string()],
                settings: TokenSettings {
                    foreground: Some("#B5CEA8".to_string()),
                    background: None,
                    font_style: None,
                },
            },
            TokenColor {
                name: Some("Function".to_string()),
                scope: vec!["entity.name.function".to_string(), "support.function".to_string()],
                settings: TokenSettings {
                    foreground: Some("#DCDCAA".to_string()),
                    background: None,
                    font_style: None,
                },
            },
            TokenColor {
                name: Some("Class".to_string()),
                scope: vec!["entity.name.type".to_string(), "entity.name.class".to_string()],
                settings: TokenSettings {
                    foreground: Some("#4EC9B0".to_string()),
                    background: None,
                    font_style: None,
                },
            },
            TokenColor {
                name: Some("Variable".to_string()),
                scope: vec!["variable".to_string()],
                settings: TokenSettings {
                    foreground: Some("#9CDCFE".to_string()),
                    background: None,
                    font_style: None,
                },
            },
            TokenColor {
                name: Some("Operator".to_string()),
                scope: vec!["keyword.operator".to_string()],
                settings: TokenSettings {
                    foreground: Some("#D4D4D4".to_string()),
                    background: None,
                    font_style: None,
                },
            },
            TokenColor {
                name: Some("Type".to_string()),
                scope: vec!["entity.name.type".to_string(), "support.type".to_string()],
                settings: TokenSettings {
                    foreground: Some("#4EC9B0".to_string()),
                    background: None,
                    font_style: None,
                },
            },
            TokenColor {
                name: Some("Constant".to_string()),
                scope: vec!["constant".to_string()],
                settings: TokenSettings {
                    foreground: Some("#569CD6".to_string()),
                    background: None,
                    font_style: None,
                },
            },
        ]
    }
    
    /// Light+ token colors for syntax highlighting
    pub fn light_plus_tokens() -> Vec<Self> {
        vec![
            TokenColor {
                name: Some("Comment".to_string()),
                scope: vec!["comment".to_string()],
                settings: TokenSettings {
                    foreground: Some("#008000".to_string()),
                    background: None,
                    font_style: Some("italic".to_string()),
                },
            },
            TokenColor {
                name: Some("Keyword".to_string()),
                scope: vec!["keyword".to_string(), "storage.type".to_string(), "storage.modifier".to_string()],
                settings: TokenSettings {
                    foreground: Some("#0000FF".to_string()),
                    background: None,
                    font_style: None,
                },
            },
            TokenColor {
                name: Some("String".to_string()),
                scope: vec!["string".to_string()],
                settings: TokenSettings {
                    foreground: Some("#A31515".to_string()),
                    background: None,
                    font_style: None,
                },
            },
            TokenColor {
                name: Some("Number".to_string()),
                scope: vec!["constant.numeric".to_string()],
                settings: TokenSettings {
                    foreground: Some("#098658".to_string()),
                    background: None,
                    font_style: None,
                },
            },
            TokenColor {
                name: Some("Function".to_string()),
                scope: vec!["entity.name.function".to_string(), "support.function".to_string()],
                settings: TokenSettings {
                    foreground: Some("#795E26".to_string()),
                    background: None,
                    font_style: None,
                },
            },
            TokenColor {
                name: Some("Class".to_string()),
                scope: vec!["entity.name.type".to_string(), "entity.name.class".to_string()],
                settings: TokenSettings {
                    foreground: Some("#267F99".to_string()),
                    background: None,
                    font_style: None,
                },
            },
            TokenColor {
                name: Some("Variable".to_string()),
                scope: vec!["variable".to_string()],
                settings: TokenSettings {
                    foreground: Some("#001080".to_string()),
                    background: None,
                    font_style: None,
                },
            },
            TokenColor {
                name: Some("Operator".to_string()),
                scope: vec!["keyword.operator".to_string()],
                settings: TokenSettings {
                    foreground: Some("#000000".to_string()),
                    background: None,
                    font_style: None,
                },
            },
            TokenColor {
                name: Some("Type".to_string()),
                scope: vec!["entity.name.type".to_string(), "support.type".to_string()],
                settings: TokenSettings {
                    foreground: Some("#267F99".to_string()),
                    background: None,
                    font_style: None,
                },
            },
            TokenColor {
                name: Some("Constant".to_string()),
                scope: vec!["constant".to_string()],
                settings: TokenSettings {
                    foreground: Some("#0000FF".to_string()),
                    background: None,
                    font_style: None,
                },
            },
        ]
    }
    
    /// High contrast token colors
    pub fn high_contrast_tokens() -> Vec<Self> {
        vec![
            TokenColor {
                name: Some("Comment".to_string()),
                scope: vec!["comment".to_string()],
                settings: TokenSettings {
                    foreground: Some("#7CA668".to_string()),
                    background: None,
                    font_style: Some("italic".to_string()),
                },
            },
            TokenColor {
                name: Some("Keyword".to_string()),
                scope: vec!["keyword".to_string()],
                settings: TokenSettings {
                    foreground: Some("#569CD6".to_string()),
                    background: None,
                    font_style: Some("bold".to_string()),
                },
            },
            TokenColor {
                name: Some("String".to_string()),
                scope: vec!["string".to_string()],
                settings: TokenSettings {
                    foreground: Some("#CE9178".to_string()),
                    background: None,
                    font_style: None,
                },
            },
            TokenColor {
                name: Some("Function".to_string()),
                scope: vec!["entity.name.function".to_string()],
                settings: TokenSettings {
                    foreground: Some("#DCDCAA".to_string()),
                    background: None,
                    font_style: Some("bold".to_string()),
                },
            },
        ]
    }
}

impl SemanticColors {
    /// Default semantic colors for dark themes
    pub fn default_dark() -> Self {
        let mut rules = HashMap::new();
        
        rules.insert("type".to_string(), SemanticColorRule {
            foreground: Some("#4EC9B0".to_string()),
            background: None,
            font_style: None,
            bold: None,
            italic: None,
            underline: None,
            strikethrough: None,
        });
        
        rules.insert("function".to_string(), SemanticColorRule {
            foreground: Some("#DCDCAA".to_string()),
            background: None,
            font_style: None,
            bold: None,
            italic: None,
            underline: None,
            strikethrough: None,
        });
        
        rules.insert("variable".to_string(), SemanticColorRule {
            foreground: Some("#9CDCFE".to_string()),
            background: None,
            font_style: None,
            bold: None,
            italic: None,
            underline: None,
            strikethrough: None,
        });
        
        Self { enabled: true, rules }
    }
    
    /// Default semantic colors for light themes
    pub fn default_light() -> Self {
        let mut rules = HashMap::new();
        
        rules.insert("type".to_string(), SemanticColorRule {
            foreground: Some("#267F99".to_string()),
            background: None,
            font_style: None,
            bold: None,
            italic: None,
            underline: None,
            strikethrough: None,
        });
        
        rules.insert("function".to_string(), SemanticColorRule {
            foreground: Some("#795E26".to_string()),
            background: None,
            font_style: None,
            bold: None,
            italic: None,
            underline: None,
            strikethrough: None,
        });
        
        rules.insert("variable".to_string(), SemanticColorRule {
            foreground: Some("#001080".to_string()),
            background: None,
            font_style: None,
            bold: None,
            italic: None,
            underline: None,
            strikethrough: None,
        });
        
        Self { enabled: true, rules }
    }
    
    /// High contrast dark semantic colors
    pub fn high_contrast_dark() -> Self {
        let mut rules = HashMap::new();
        
        rules.insert("type".to_string(), SemanticColorRule {
            foreground: Some("#4EC9B0".to_string()),
            background: None,
            font_style: None,
            bold: Some(true),
            italic: None,
            underline: None,
            strikethrough: None,
        });
        
        rules.insert("function".to_string(), SemanticColorRule {
            foreground: Some("#DCDCAA".to_string()),
            background: None,
            font_style: None,
            bold: Some(true),
            italic: None,
            underline: None,
            strikethrough: None,
        });
        
        Self { enabled: true, rules }
    }
    
    /// High contrast light semantic colors
    pub fn high_contrast_light() -> Self {
        let mut rules = HashMap::new();
        
        rules.insert("type".to_string(), SemanticColorRule {
            foreground: Some("#267F99".to_string()),
            background: None,
            font_style: None,
            bold: Some(true),
            italic: None,
            underline: None,
            strikethrough: None,
        });
        
        rules.insert("function".to_string(), SemanticColorRule {
            foreground: Some("#795E26".to_string()),
            background: None,
            font_style: None,
            bold: Some(true),
            italic: None,
            underline: None,
            strikethrough: None,
        });
        
        Self { enabled: true, rules }
    }
}