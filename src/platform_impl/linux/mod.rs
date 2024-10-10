// Copyright 2022-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use crate::hotkey::HotKey;

#[path = "x11/mod.rs"]
mod x11;

#[path = "wayland/mod.rs"]
mod wayland;

pub(crate) enum GlobalHotKeyManager {
    X11(x11::GlobalHotKeyManager),
    Wayland(wayland::GlobalHotKeyManager),
}

impl GlobalHotKeyManager {
    pub(crate) fn new() -> crate::Result<Self> {
        match std::env::var("XDG_SESSION_TYPE") {
            Ok(env) => {
                let env_str = env.as_str();
                match env_str {
                    "x11" => x11::GlobalHotKeyManager::new().map(GlobalHotKeyManager::X11),

                    "wayland" => wayland::GlobalHotKeyManager::new().map(GlobalHotKeyManager::Wayland),
                    _ => {
                        let error = std::io::Error::new(std::io::ErrorKind::NotFound, format!("Unknown XDG_SESSION_TYPE: {}, expected x11 or wayland.", env_str));
                        Err(crate::Error::OsError(error))
                    },
                }
            },
            Err(e) => {
                let error = std::io::Error::new(std::io::ErrorKind::Other, e);
                Err(crate::Error::OsError(error))
            },
        }
    }
    pub(crate) fn register(&self, hotkey: HotKey) -> crate::Result<()> {
        match self {
            GlobalHotKeyManager::Wayland(wayland) => {wayland.register(hotkey)},
            GlobalHotKeyManager::X11(x11) => {x11.register(hotkey)},
        }
    }

    pub(crate) fn unregister(&self, hotkey: HotKey) -> crate::Result<()> {
        match self {
            GlobalHotKeyManager::Wayland(wayland) => {wayland.unregister(hotkey)},
            GlobalHotKeyManager::X11(x11) => {x11.unregister(hotkey)},
        }
    }
    pub(crate) fn register_all(&self, hotkeys: &[HotKey]) -> crate::Result<()> {
        match self {
            GlobalHotKeyManager::Wayland(wayland) => {wayland.register_all(hotkeys)},
            GlobalHotKeyManager::X11(x11) => {x11.register_all(hotkeys)},
        }
    }

    pub(crate) fn unregister_all(&self, hotkeys: &[HotKey]) -> crate::Result<()> {
        match self {
            GlobalHotKeyManager::Wayland(wayland) => {wayland.unregister_all(hotkeys)},
            GlobalHotKeyManager::X11(x11) => {x11.unregister_all(hotkeys)},
        }
    }

}