//! Handing the window to a `UIScene`, which iOS 13 and later require.
//!
//! `winit` 0.30 still drives iOS through the pre‑iOS 13 `UIApplicationDelegate`
//! path: it builds a bare `UIWindow` and sends it `makeKeyAndVisible`. Two
//! things go wrong on a modern iPhone.
//!
//! The first is fatal. Since iOS 26, `UIKit` traps at launch — `brk 0` inside
//! `__UIApplicationEvaluateRuntimeIssueForNoSceneLifecycleAdoption` — for any
//! app that declares no scene manifest and is linked against the new SDK.
//! `ios/Info.plist` now declares one, which stops the trap.
//!
//! The second is quieter: once an app adopts the scene lifecycle, a `UIWindow`
//! belonging to no `UIWindowScene` is never shown. The app runs, the event loop
//! spins, every frame is rasterised and presented — onto nothing. The screen
//! stays black, and nothing in the logs says why.
//!
//! This module closes that second half. Note that the window has to be reached
//! through winit's own handle: `UIApplication::windows` enumerates the windows
//! of *connected scenes*, so a scene‑less window is precisely the thing it
//! cannot see.

// The rest of the crate is unsafe‑free; reconstructing a `UIView` from a raw
// pointer is not, so the exception is confined to this file.
#![allow(unsafe_code)]

use objc2::ClassType;
use objc2::rc::Retained;
use objc2::runtime::NSObjectProtocol;
use objc2_foundation::MainThreadMarker;
use objc2_ui_kit::{UIApplication, UIView, UIWindowScene};
use winit::raw_window_handle::{HasWindowHandle, RawWindowHandle};

/// Puts `window` on the first `UIWindowScene` `UIKit` has connected.
///
/// Idempotent and cheap: a window that already belongs to a scene is left
/// alone, so this can run on every resume.
pub fn adopt(window: &winit::window::Window) {
    // UIKit is main-thread only. winit calls us from the event loop, which is
    // the main thread, but prove it rather than assume it.
    let Some(mtm) = MainThreadMarker::new() else {
        return;
    };
    let Some(view) = view_of(window) else {
        return;
    };
    // `UIView::window` is the owning `UIWindow`, which is the one winit made.
    let Some(ui_window) = view.window() else {
        return;
    };

    // SAFETY: main thread, and `ui_window` is a live UIKit object.
    if unsafe { ui_window.windowScene() }.is_some() {
        return;
    }
    let Some(scene) = first_window_scene(&UIApplication::sharedApplication(mtm)) else {
        return;
    };
    // SAFETY: as above; this is the assignment the whole module exists for.
    unsafe { ui_window.setWindowScene(Some(&scene)) };
}

/// winit's backing `UIView`, borrowed from the raw window handle.
fn view_of(window: &winit::window::Window) -> Option<Retained<UIView>> {
    let handle = window.window_handle().ok()?;
    let RawWindowHandle::UiKit(handle) = handle.as_raw() else {
        return None;
    };
    let ptr = handle.ui_view.as_ptr().cast::<UIView>();
    // SAFETY: winit guarantees `ui_view` points at a live `UIView` for as long
    // as the `Window` it came from is alive, and `window` is borrowed here.
    let view = unsafe { ptr.as_ref() }?;
    Some(view.retain())
}

/// The first connected scene that is a `UIWindowScene`, if `UIKit` made one.
fn first_window_scene(app: &UIApplication) -> Option<Retained<UIWindowScene>> {
    // SAFETY: main thread; the set is owned by us for the duration of the call.
    let scenes = unsafe { app.connectedScenes() };
    // SAFETY: same; `allObjects` just flattens the set into an array.
    let scenes = unsafe { scenes.allObjects() };

    scenes.iter().find_map(|scene| {
        if !scene.isKindOfClass(UIWindowScene::class()) {
            return None;
        }
        // SAFETY: checked immediately above.
        Some(unsafe { Retained::cast::<UIWindowScene>(scene.retain()) })
    })
}
