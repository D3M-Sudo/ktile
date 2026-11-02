//! Read-only X11/EWMH discovery adapter.
//!
//! This module intentionally does not issue geometry or focus requests. Window
//! mutation belongs to a later milestone after reconciliation and restoration
//! behavior have dedicated integration tests.

use crate::{layout::Rect, model::{WindowId, WindowState}};
use thiserror::Error;
use x11rb::{
    connection::Connection,
    protocol::xproto::{Atom, AtomEnum, ConnectionExt, Window},
    rust_connection::RustConnection,
};

#[derive(Debug, Error)]
pub enum X11Error {
    #[error("cannot connect to X11: {0}")]
    Connect(#[from] x11rb::errors::ConnectError),
    #[error("X11 connection error: {0}")]
    Connection(#[from] x11rb::errors::ConnectionError),
    #[error("X11 reply error: {0}")]
    Reply(#[from] x11rb::errors::ReplyError),
    #[error("no EWMH-compliant window manager detected")]
    NoWindowManager,
}

pub struct X11Backend {
    connection: RustConnection,
    screen_num: usize,
}

impl X11Backend {
    pub fn connect() -> Result<Self, X11Error> {
        let (connection, screen_num) = x11rb::connect(None)?;
        let screen = &connection.setup().roots[screen_num];
        let root = screen.root;
        let supporting_atom = Self::intern(&connection, b"_NET_SUPPORTING_WM_CHECK")?;
        let property = connection
            .get_property(false, root, supporting_atom, AtomEnum::WINDOW, 0, 1)?
            .reply()?;
        let wm_window = property
            .value32()
            .and_then(|mut values| values.next())
            .ok_or(X11Error::NoWindowManager)?;

        // EWMH requires the supporting-WM property on the WM window to point
        // back to itself. A root property alone is not sufficient evidence.
        let wm_property = connection
            .get_property(false, wm_window, supporting_atom, AtomEnum::WINDOW, 0, 1)?
            .reply()?;
        if wm_property.value32().and_then(|mut values| values.next()) != Some(wm_window) {
            return Err(X11Error::NoWindowManager);
        }

        Ok(Self {
            connection,
            screen_num,
        })
    }

    fn intern(connection: &RustConnection, name: &[u8]) -> Result<Atom, X11Error> {
        Ok(connection.intern_atom(false, name)?.reply()?.atom)
    }

    pub fn screen_size(&self) -> (u16, u16) {
        let screen = &self.connection.setup().roots[self.screen_num];
        (screen.width_in_pixels, screen.height_in_pixels)
    }

    pub fn root_window(&self) -> Window {
        self.connection.setup().roots[self.screen_num].root
    }

    /// Read the current EWMH desktop index, if the window manager publishes it.
    pub fn current_desktop(&self) -> Result<Option<u32>, X11Error> {
        let atom = Self::intern(&self.connection, b"_NET_CURRENT_DESKTOP")?;
        let reply = self
            .connection
            .get_property(false, self.root_window(), atom, AtomEnum::CARDINAL, 0, 1)?
            .reply()?;
        Ok(reply.value32().and_then(|mut values| values.next()))
    }

    /// Read the number of virtual desktops, if published by the window manager.
    pub fn desktop_count(&self) -> Result<Option<u32>, X11Error> {
        let atom = Self::intern(&self.connection, b"_NET_NUMBER_OF_DESKTOPS")?;
        let reply = self
            .connection
            .get_property(false, self.root_window(), atom, AtomEnum::CARDINAL, 0, 1)?
            .reply()?;
        Ok(reply.value32().and_then(|mut values| values.next()))
    }

    /// Read the EWMH work area for one desktop, including reserved panel space.
    pub fn work_area(&self, desktop: u32) -> Result<Option<Rect>, X11Error> {
        let Some(offset) = desktop.checked_mul(4) else {
            return Ok(None);
        };
        let atom = Self::intern(&self.connection, b"_NET_WORKAREA")?;
        let reply = self
            .connection
            .get_property(
                false,
                self.root_window(),
                atom,
                AtomEnum::CARDINAL,
                offset,
                4,
            )?
            .reply()?;
        let values: Vec<u32> = reply
            .value32()
            .map(|values| values.collect())
            .unwrap_or_default();
        if values.len() != 4 {
            return Ok(None);
        }
        let area = Rect {
            x: values[0] as i32,
            y: values[1] as i32,
            width: values[2],
            height: values[3],
        };
        Ok(area.is_valid().then_some(area))
    }

    /// Read the EWMH client list without changing any desktop state.
    pub fn client_windows(&self) -> Result<Vec<WindowId>, X11Error> {
        let atom = Self::intern(&self.connection, b"_NET_CLIENT_LIST")?;
        let reply = self
            .connection
            .get_property(
                false,
                self.root_window(),
                atom,
                AtomEnum::WINDOW,
                0,
                u32::MAX,
            )?
            .reply()?;
        Ok(reply
            .value32()
            .map(|values| values.map(WindowId).collect())
            .unwrap_or_default())
    }

    /// Read a window title, preferring UTF-8 EWMH title over legacy WM_NAME.
    pub fn window_title(&self, window: WindowId) -> Result<Option<String>, X11Error> {
        let name_atom = Self::intern(&self.connection, b"_NET_WM_NAME")?;
        let utf8_atom = Self::intern(&self.connection, b"UTF8_STRING")?;
        let modern = self
            .connection
            .get_property(false, window.0, name_atom, utf8_atom, 0, 16_384)?
            .reply()?;
        if !modern.value.is_empty() {
            return Ok(Some(String::from_utf8_lossy(&modern.value).into_owned()));
        }

        let legacy = self
            .connection
            .get_property(
                false,
                window.0,
                AtomEnum::WM_NAME,
                AtomEnum::STRING,
                0,
                16_384,
            )?
            .reply()?;
        if legacy.value.is_empty() {
            Ok(None)
        } else {
            Ok(Some(String::from_utf8_lossy(&legacy.value).into_owned()))
        }
    }

    /// WM_CLASS is two NUL-separated strings: instance followed by class.
    pub fn window_class(&self, window: WindowId) -> Result<Option<String>, X11Error> {
        let reply = self
            .connection
            .get_property(
                false,
                window.0,
                AtomEnum::WM_CLASS,
                AtomEnum::STRING,
                0,
                1024,
            )?
            .reply()?;
        if reply.value.is_empty() {
            return Ok(None);
        }
        let parts: Vec<_> = reply
            .value
            .split(|byte| *byte == 0)
            .filter(|part| !part.is_empty())
            .collect();
        let class = parts.get(1).or_else(|| parts.first());
        Ok(class.map(|value| String::from_utf8_lossy(value).into_owned()))
    }

    pub fn window_desktop(&self, window: WindowId) -> Result<Option<u32>, X11Error> {
        let atom = Self::intern(&self.connection, b"_NET_WM_DESKTOP")?;
        let reply = self
            .connection
            .get_property(false, window.0, atom, AtomEnum::CARDINAL, 0, 1)?
            .reply()?;
        Ok(reply.value32().and_then(|mut values| values.next()))
    }

    pub fn window_state(&self, window: WindowId) -> Result<WindowState, X11Error> {
        let state_atom = Self::intern(&self.connection, b"_NET_WM_STATE")?;
        let reply = self
            .connection
            .get_property(false, window.0, state_atom, AtomEnum::ATOM, 0, 64)?
            .reply()?;
        let states: Vec<Atom> = reply
            .value32()
            .map(|values| values.collect())
            .unwrap_or_default();
        let has = |name: &[u8]| -> Result<bool, X11Error> {
            let atom = Self::intern(&self.connection, name)?;
            Ok(states.contains(&atom))
        };
        Ok(WindowState {
            fullscreen: has(b"_NET_WM_STATE_FULLSCREEN")?,
            maximized_horizontal: has(b"_NET_WM_STATE_MAXIMIZED_HORZ")?,
            maximized_vertical: has(b"_NET_WM_STATE_MAXIMIZED_VERT")?,
            modal: has(b"_NET_WM_STATE_MODAL")?,
            above: has(b"_NET_WM_STATE_ABOVE")?,
            below: has(b"_NET_WM_STATE_BELOW")?,
            skip_taskbar: has(b"_NET_WM_STATE_SKIP_TASKBAR")?,
            skip_pager: has(b"_NET_WM_STATE_SKIP_PAGER")?,
            sticky: has(b"_NET_WM_STATE_STICKY")?,
            minimized: has(b"_NET_WM_STATE_HIDDEN")?,
        })
    }
}
