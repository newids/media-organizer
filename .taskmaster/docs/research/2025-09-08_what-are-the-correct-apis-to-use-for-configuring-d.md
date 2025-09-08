---
title: Research Session
query: "What are the correct APIs to use for configuring Dioxus Desktop 0.6.x applications with window configuration and menu bars? How to use launch_with_props, use_window, or WindowBuilder with dioxus 0.6.3?"
date: 9/8/2025
time: 1:52:42 PM
timestamp: 2025-09-08T04:52:42.331Z
exchanges: 1
---

# Research Session

## Initial Query

**Question:** What are the correct APIs to use for configuring Dioxus Desktop 0.6.x applications with window configuration and menu bars? How to use launch_with_props, use_window, or WindowBuilder with dioxus 0.6.3?

**Response:**

For **Dioxus Desktop 0.6.x** (specifically 0.6.3), the correct APIs for configuring window properties and menu bars are primarily exposed through the `dioxus_desktop` crate, using the `launch_with_props`, `use_window`, and `WindowBuilder` patterns. However, menu bar integration is currently limited and typically requires direct use of lower-level libraries (`wry` and `tao`) rather than high-level Dioxus abstractions[5][1].

---

## Window Configuration APIs

### 1. **`launch_with_props`**
- **Purpose:** Launches your Dioxus app with custom configuration.
- **Usage:** Pass a `DesktopConfig` struct to configure window size, title, resizability, and other properties.
- **Example:**
  ```rust
  use dioxus::prelude::*;
  use dioxus_desktop::{launch_with_props, DesktopConfig};

  fn main() {
      launch_with_props(
          app,
          DesktopConfig {
              window: Some(
                  dioxus_desktop::WindowBuilder::new()
                      .with_title("My App")
                      .with_inner_size((800, 600).into())
                      .with_resizable(true)
                      .build()
              ),
              ..Default::default()
          }
      );
  }

  fn app(cx: Scope) -> Element {
      cx.render(rsx! { div { "Hello, world!" } })
  }
  ```
- **Notes:** `DesktopConfig` allows you to set window properties and other desktop-specific options[1][5].

### 2. **`WindowBuilder`**
- **Purpose:** Used to construct and customize the window before launching.
- **Usage:** Chain methods to set window title, size, position, resizability, etc.
- **Example:**
  ```rust
  let window = dioxus_desktop::WindowBuilder::new()
      .with_title("My App")
      .with_inner_size((1024, 768).into())
      .with_resizable(false)
      .build();
  ```
- **Integration:** Pass the built window to `DesktopConfig` as shown above[1][5].

### 3. **`use_window` Hook**
- **Purpose:** Access and manipulate the window at runtime from within your components.
- **Usage:** Call `use_window()` inside your component to get a handle to the window.
- **Example:**
  ```rust
  use dioxus_desktop::use_window;

  fn app(cx: Scope) -> Element {
      let window = use_window(cx);
      // You can now call methods on the window, e.g., set title, resize, etc.
      cx.render(rsx! { div { "Window handle available!" } })
  }
  ```
- **Notes:** Useful for dynamic window changes (e.g., responding to user actions)[1].

---

## Menu Bar Integration

- **Current State:** Dioxus Desktop 0.6.x does **not** provide high-level APIs for menu bars. You must use the underlying `wry` and `tao` APIs directly for native menu integration[5].
- **Actionable Steps:**
  - Access the window handle via `use_window` or during configuration.
  - Use `tao::menu` or `wry` APIs to create and attach menus.
  - Example (pseudo-code, not Dioxus-specific):
    ```rust
    // After getting the window handle
    let menu = tao::menu::Menu::new();
    // Add menu items, then attach to window
    window.set_menu(menu);
    ```
- **Future:** Dioxus plans to add higher-level menu APIs in upcoming releases[5].

---

## Recommendations for Your Project

Given your project's focus on **VS Code-like layout, cross-platform compatibility, and accessibility**, you should:

- Use **`launch_with_props`** and **`WindowBuilder`** for initial window configuration (size, title, resizability).
- Use **`use_window`** for runtime window manipulation (e.g., responding to settings changes or accessibility events).
- For **menu bars**, implement them using `tao` directly until Dioxus provides native support. This ensures cross-platform consistency, especially for macOS and Windows[5].
- Document any direct `tao`/`wry` usage for future migration to Dioxus-native APIs.

---

## Example: Minimal Dioxus Desktop 0.6.3 Setup

```rust
use dioxus::prelude::*;
use dioxus_desktop::{launch_with_props, DesktopConfig, WindowBuilder};

fn main() {
    launch_with_props(
        app,
        DesktopConfig {
            window: Some(
                WindowBuilder::new()
                    .with_title("VS Code Clone")
                    .with_inner_size((1200, 800).into())
                    .with_resizable(true)
                    .build()
            ),
            ..Default::default()
        }
    );
}

fn app(cx: Scope) -> Element {
    let window = dioxus_desktop::use_window(cx);
    // Use window for runtime changes if needed
    cx.render(rsx! { div { "Editor UI goes here" } })
}
```
---

**Summary of APIs:**
- **Window config:** `launch_with_props`, `DesktopConfig`, `WindowBuilder`
- **Runtime window access:** `use_window`
- **Menu bars:** Use `tao`/`wry` directly for now[1][5]

This approach aligns with your project's requirements for UI density, layout control, and cross-platform support. For future-proofing, monitor Dioxus releases for improved menu and desktop integration.


---

*Generated by Task Master Research Command*  
*Timestamp: 2025-09-08T04:52:42.331Z*
