// Copyright 2022-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

mod runtime;

use std::collections::BTreeMap;
use ashpd::Error;
use crossbeam_channel::{unbounded, Receiver, Sender};
use keyboard_types::{Code, Modifiers};
use crate::{hotkey::HotKey, GlobalHotKeyEvent};
use tokio::runtime::Runtime;
use crate::platform_impl::platform::wayland::runtime::Xdgs;

enum ThreadMessage {
    RegisterHotKey(HotKey, Sender<crate::Result<()>>),
    RegisterHotKeys(Vec<HotKey>, Sender<crate::Result<()>>),
    UnRegisterHotKey(HotKey, Sender<crate::Result<()>>),
    UnRegisterHotKeys(Vec<HotKey>, Sender<crate::Result<()>>),
    DropThread,
}

pub struct GlobalHotKeyManager {
    thread_tx: Sender<ThreadMessage>,
}

impl GlobalHotKeyManager {
    pub fn new() -> crate::Result<Self> {
        let (thread_tx, thread_rx) = unbounded();
        std::thread::spawn(|| events_processor(thread_rx));
        Ok(Self { thread_tx })
    }

    pub fn register(&self, hotkey: HotKey) -> crate::Result<()> {
        let (tx, rx) = crossbeam_channel::bounded(1);
        let _ = self
            .thread_tx
            .send(ThreadMessage::RegisterHotKey(hotkey, tx));

        if let Ok(result) = rx.recv() {
            result?;
        }

        Ok(())
    }

    pub fn unregister(&self, hotkey: HotKey) -> crate::Result<()> {
        let (tx, rx) = crossbeam_channel::bounded(1);
        let _ = self
            .thread_tx
            .send(ThreadMessage::UnRegisterHotKey(hotkey, tx));

        if let Ok(result) = rx.recv() {
            result?;
        }

        Ok(())
    }

    pub fn register_all(&self, hotkeys: &[HotKey]) -> crate::Result<()> {
        let (tx, rx) = crossbeam_channel::bounded(1);
        let _ = self
            .thread_tx
            .send(ThreadMessage::RegisterHotKeys(hotkeys.to_vec(), tx));

        if let Ok(result) = rx.recv() {
            result?;
        }

        Ok(())
    }

    pub fn unregister_all(&self, hotkeys: &[HotKey]) -> crate::Result<()> {
        let (tx, rx) = crossbeam_channel::bounded(1);
        let _ = self
            .thread_tx
            .send(ThreadMessage::UnRegisterHotKeys(hotkeys.to_vec(), tx));

        if let Ok(result) = rx.recv() {
            result?;
        }

        Ok(())
    }
}

impl Drop for GlobalHotKeyManager {
    fn drop(&mut self) {
        let _ = self.thread_tx.send(ThreadMessage::DropThread);
    }
}

#[inline]
fn register_hotkey(
    xdgs: &Xdgs,
    hotkeys: &mut Vec<u32>,
    hotkey: HotKey,
) -> crate::Result<()> {
    let (modifiers, key) = (
        modifiers_to_freedesktop_spec(hotkey.mods),
        keycode_to_freedesktop_spec(hotkey.key),
    );

    if let Some(key) = key {
        let xdg_shortcut  = format!("{}+{}", modifiers, key);
        xdgs.register(xdg_shortcut, hotkeys)
    } else {
        Err(crate::Error::FailedToRegister(format!(
            "Unable to register accelerator (unknown scancode for this key: {}).",
            hotkey.key
        )))
    }
}

#[inline]
fn unregister_hotkey(
    xdgs: &Xdgs,
    hotkeys: &mut Vec<u32>,
    hotkey: HotKey,
) -> crate::Result<()> {
    let (modifiers, key) = (
        modifiers_to_freedesktop_spec(hotkey.mods),
        keycode_to_freedesktop_spec(hotkey.key),
    );

    if let Some(key) = key {
        let xdg_shortcut = format!("{}+{}", modifiers, key);
        xdgs.unregister(xdg_shortcut, hotkey, hotkeys)
    } else {
        Err(crate::Error::FailedToUnRegister(hotkey))
    }
}

fn events_processor(thread_rx: Receiver<ThreadMessage>) {
    let mut hotkeys: Vec<u32> = Vec::new();
    if let Ok(xdg) = Xdgs::new() {
        loop {
            if let Ok(msg) = thread_rx.try_recv() {
                match msg {
                    ThreadMessage::RegisterHotKey(hotkey, tx) => {
                        let _ = tx.send(register_hotkey(
                            &xdg,
                            &mut hotkeys,
                            hotkey,
                        ));
                    }
                    ThreadMessage::RegisterHotKeys(keys, tx) => {
                        for hotkey in keys {
                            if let Err(e) =
                                register_hotkey(&xdg, &mut hotkeys, hotkey)
                            {
                                let _ = tx.send(Err(e));
                            }
                        }
                        let _ = tx.send(Ok(()));
                    }
                    ThreadMessage::UnRegisterHotKey(hotkey, tx) => {
                        let _ = tx.send(unregister_hotkey(
                            &xdg,
                            &mut hotkeys,
                            hotkey,
                        ));
                    }
                    ThreadMessage::UnRegisterHotKeys(keys, tx) => {
                        for hotkey in keys {
                            if let Err(e) =
                                unregister_hotkey(&xdg, &mut hotkeys, hotkey)
                            {
                                let _ = tx.send(Err(e));
                            }
                        }
                        let _ = tx.send(Ok(()));
                    }
                    ThreadMessage::DropThread => {
                        (drop(xdg));
                        return;
                    }
                }
            }
            xdg.activated();
            xdg.deactivated();

            std::thread::sleep(std::time::Duration::from_millis(50));
        }
    } else {
        #[cfg(debug_assertions)]
        eprintln!("Failed to open global shortcut portal, it might not be implemented on your desktop environment. The portal is required for global-hotkey crate under wayland try x11 instead.");
    }
}


// keycode_to_xdg_spec and modifiers_to_x11_mods are simple mappings from the `keyboard-types` crate to the freedesktop specification.
// see https://specifications.freedesktop.org/shortcuts-spec/latest/
fn keycode_to_freedesktop_spec(key: Code) -> Option<String> {
    Some(match key {
        Code::KeyA => "A".to_string(),
        Code::KeyB => "B".to_string(),
        Code::KeyC => "C".to_string(),
        Code::KeyD => "D".to_string(),
        Code::KeyE => "E".to_string(),
        Code::KeyF => "F".to_string(),
        Code::KeyG => "G".to_string(),
        Code::KeyH => "H".to_string(),
        Code::KeyI => "I".to_string(),
        Code::KeyJ => "J".to_string(),
        Code::KeyK => "K".to_string(),
        Code::KeyL => "L".to_string(),
        Code::KeyM => "M".to_string(),
        Code::KeyN => "N".to_string(),
        Code::KeyO => "O".to_string(),
        Code::KeyP => "P".to_string(),
        Code::KeyQ => "Q".to_string(),
        Code::KeyR => "R".to_string(),
        Code::KeyS => "S".to_string(),
        Code::KeyT => "T".to_string(),
        Code::KeyU => "U".to_string(),
        Code::KeyV => "V".to_string(),
        Code::KeyW => "W".to_string(),
        Code::KeyX => "X".to_string(),
        Code::KeyY => "Y".to_string(),
        Code::KeyZ => "Z".to_string(),
        Code::Backslash => "backslash".to_string(),
        Code::BracketLeft => "bracketleft".to_string(),
        Code::BracketRight => "bracketright".to_string(),
        Code::Backquote => "quoteleft".to_string(),
        Code::Comma => "comma".to_string(),
        Code::Digit0 => "0".to_string(),
        Code::Digit1 => "1".to_string(),
        Code::Digit2 => "2".to_string(),
        Code::Digit3 => "3".to_string(),
        Code::Digit4 => "4".to_string(),
        Code::Digit5 => "5".to_string(),
        Code::Digit6 => "6".to_string(),
        Code::Digit7 => "7".to_string(),
        Code::Digit8 => "8".to_string(),
        Code::Digit9 => "9".to_string(),
        Code::Equal => "equal".to_string(),
        Code::Minus => "minus".to_string(),
        Code::Period => "period".to_string(),
        Code::Quote => "leftsinglequotemark".to_string(),
        Code::Semicolon => "semicolon".to_string(),
        Code::Slash => "slash".to_string(),
        Code::Backspace => "BackSpace".to_string(),
        Code::CapsLock => "Caps_Lock".to_string(),
        Code::Enter => "Return".to_string(),
        Code::Space => "space".to_string(),
        Code::Tab => "Tab".to_string(),
        Code::Delete => "Delete".to_string(),
        Code::End => "End".to_string(),
        Code::Home => "Home".to_string(),
        Code::Insert => "Insert".to_string(),
        Code::PageDown => "Page_Down".to_string(),
        Code::PageUp => "Page_Up".to_string(),
        Code::ArrowDown => "Down".to_string(),
        Code::ArrowLeft => "Left".to_string(),
        Code::ArrowRight => "Right".to_string(),
        Code::ArrowUp => "Up".to_string(),
        Code::Numpad0 => "KP_0".to_string(),
        Code::Numpad1 => "KP_1".to_string(),
        Code::Numpad2 => "KP_2".to_string(),
        Code::Numpad3 => "KP_3".to_string(),
        Code::Numpad4 => "KP_4".to_string(),
        Code::Numpad5 => "KP_5".to_string(),
        Code::Numpad6 => "KP_6".to_string(),
        Code::Numpad7 => "KP_7".to_string(),
        Code::Numpad8 => "KP_8".to_string(),
        Code::Numpad9 => "KP_9".to_string(),
        Code::NumpadAdd => "KP_Add".to_string(),
        Code::NumpadDecimal => "KP_Decimal".to_string(),
        Code::NumpadDivide => "KP_Divide".to_string(),
        Code::NumpadMultiply => "KP_Multiply".to_string(),
        Code::NumpadSubtract => "KP_Subtract".to_string(),
        Code::Escape => "Escape".to_string(),
        Code::PrintScreen => "Print".to_string(),
        Code::ScrollLock => "Scroll_Lock".to_string(),
        Code::NumLock => "F1".to_string(),
        Code::F1 => "F1".to_string(),
        Code::F2 => "F2".to_string(),
        Code::F3 => "F3".to_string(),
        Code::F4 => "F4".to_string(),
        Code::F5 => "F5".to_string(),
        Code::F6 => "F6".to_string(),
        Code::F7 => "F7".to_string(),
        Code::F8 => "F8".to_string(),
        Code::F9 => "F9".to_string(),
        Code::F10 => "F10".to_string(),
        Code::F11 => "F11".to_string(),
        Code::F12 => "F12".to_string(),
        Code::AudioVolumeDown => "XF86AudioLowerVolume".to_string(),
        Code::AudioVolumeMute => "XF86XK_AudioMute".to_string(),
        Code::AudioVolumeUp => "XF86XK_AudioRaiseVolume".to_string(),
        Code::MediaPlay => "XF86XK_AudioPlay".to_string(),
        Code::MediaPause => "XF86XK_AudioPause".to_string(),
        Code::MediaStop => "XF86XK_AudioStop".to_string(),
        Code::MediaTrackNext => "XF86XK_AudioNext".to_string(),
        Code::MediaTrackPrevious => "XF86XK_AudioPrev".to_string(),
        _ => return None,
    })
}

fn modifiers_to_freedesktop_spec(modifiers: Modifiers) -> String {
    let mut xdg_mods = String::new();
    if modifiers.contains(Modifiers::SHIFT) {
        add_keys(&mut xdg_mods, "SHIFT");
    }
    if modifiers.intersects(Modifiers::SUPER | Modifiers::META) {
        add_keys(&mut xdg_mods, "LOGO");
    }
    if modifiers.contains(Modifiers::ALT) {
        add_keys(&mut xdg_mods, "ALT");
    }
    if modifiers.contains(Modifiers::CONTROL) {
        add_keys(&mut xdg_mods, "CTRL");
    }
    xdg_mods
}

fn add_keys(current: &mut String, add: &str) {
    if current.is_empty() {
        *current = add.to_string();
    } else {
        *current = format!("{}+{}", current, add);
    }
}