use crate::GlobalHotKeyEvent;
use ashpd::{desktop::{
    global_shortcuts::{
        GlobalShortcuts,
        NewShortcut,
        Activated,
        Deactivated,
    },
    Session,
}, Error, WindowIdentifier};
use ashpd::zbus::export::futures_core::Stream;
use rand::random;
use tokio::runtime::Runtime;
use tokio_stream::StreamExt;
use crate::hotkey::HotKey;
use crate::platform_impl::platform::wayland::runtime;

pub(super) struct AsyncXdgs<'a> {
    global_shortcuts: GlobalShortcuts<'a>,
    session: Session<'a, GlobalShortcuts<'a>>,
    window_identifier: WindowIdentifier,
}

impl<'a> AsyncXdgs<'a> {
    async fn new() -> Result<Self, ashpd::Error> {
        let global_shortcuts = GlobalShortcuts::new().await?;
        let session = global_shortcuts.create_session().await?;
        Ok(AsyncXdgs {
            global_shortcuts,
            session,
            window_identifier: WindowIdentifier::default(),
        })
    }

    async fn register(&self, hotkey: String, hotkeys: &mut Vec<u32>) -> Result<(), ashpd::Error> {
        let mut id;
        loop {
            id = random::<u32>();
            if hotkeys.is_empty() || !hotkeys.contains(&id) {
                break;
            } else {
                continue
            }
        }
        let shortcut = NewShortcut::new(id.clone().to_string(), "A hotkey created by the global hotkey rs library")
            .preferred_trigger(Some(hotkey.as_str()));
        let shortcuts = self.global_shortcuts.bind_shortcuts(&self.session, &[shortcut], &self.window_identifier).await?.response()?.shortcuts().to_owned();
        hotkeys.push(id);

        Ok(())
    }

    async fn unregister(&self, hotkey: String, hotkeys: &mut Vec<u32>) -> Result<(), ashpd::Error> {
        todo!()
    }

    async fn activated(&self) -> Result<(), Error> {
        match self.global_shortcuts.receive_activated().await {
            Ok(mut ok) => {
                while let Some(activated_hotkey) = ok.next().await {
                    let id = activated_hotkey.shortcut_id().parse::<u32>().expect("Failed to parse shortcut id to u32: you should never see this error because id started as a u32.");
                    GlobalHotKeyEvent::send(GlobalHotKeyEvent {
                        id,
                        state: crate::HotKeyState::Pressed,
                    });
                    break;
                }
                Ok(())
            }
            Err(err) => { Err(err) }
        }
    }

    async fn deactivated(&self) -> Result<(), Error> {
        match self.global_shortcuts.receive_deactivated().await {
            Ok(mut ok) => {
                while let Some(deactivated_hotkey) = ok.next().await {
                    let id = deactivated_hotkey.shortcut_id().parse::<u32>().expect("Failed to parse shortcut id to u32: you should never see this error because id started as a u32.");
                    GlobalHotKeyEvent::send(GlobalHotKeyEvent {
                        id,
                        state: crate::HotKeyState::Released,
                    });
                    break;
                }
                Ok(())
            }
            Err(err) => { Err(err) }
        }
    }
    async fn drop(&self) {
        let _ = self.session.close().await;
    }
}

pub(super) struct Xdgs<'a> {
    inner: AsyncXdgs<'a>,
    rt: Runtime,
}

impl<'a> Xdgs<'a> {
    pub(super) fn new() -> ashpd::Result<Self> {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_io()
            .build()?;
        match rt.block_on(runtime::AsyncXdgs::new()) {
            Ok(inner) => {
                Ok(
                    Self {
                        inner,
                        rt,
                    }
                )
            }
            Err(err) => {
                Err(ashpd::Error::NoResponse)
            }
        }
    }
    pub(super) fn register(&self, hotkey: String, hotkeys: &mut Vec<u32>) -> crate::Result<()> {
        if let Err(err) = self.rt.block_on(self.inner.register(hotkey, hotkeys)) {
            Err(crate::Error::FailedToRegister(err.to_string().into()))
        } else {
            Ok(())
        }
    }
    pub(super) fn unregister(&self, hotkey_str: String, hotkey: HotKey, hotkeys: &mut Vec<u32>) -> crate::Result<()> {
        if self.rt.block_on(self.inner.unregister(hotkey_str, hotkeys)).is_err() {
            Err(crate::Error::FailedToUnRegister(hotkey))
        } else {
            Ok(())
        }
}
    pub(super) fn activated(&self) {
        self.rt.block_on(self.inner.activated());
    }

    pub(super) fn deactivated(&self) {
        self.rt.block_on(self.inner.deactivated());
    }
}

impl<'a> Drop for Xdgs<'a> {
    fn drop(&mut self) {
        self.rt.block_on(self.inner.drop());
    }
}






