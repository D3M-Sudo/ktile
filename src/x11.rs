//! Read-only X11/EWMH discovery adapter.
//!
//! This module intentionally does not issue geometry or focus requests. Window
//! mutation belongs to a later milestone after reconciliation and restoration
//! behavior have dedicated integration tests.

use crate::{
    layout::Rect,
    model::{WindowId, WindowState},
};
use thiserror::Error;
use x11rb::{
    connection::Connection,
    protocol::randr::ConnectionExt as RandrConnectionExt,
    protocol::{
        Event,
        randr::NotifyMask,
        xproto::{Atom, AtomEnum, ChangeWindowAttributesAux, ConnectionExt, EventMask, Window},
    },
    rust_connection::RustConnection,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Monitor {
    pub name: String,
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
    pub primary: bool,
}

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
    #[error("RandR 1.5 monitor enumeration is not supported by this X server")]
    RandrUnsupported,
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

    /// Enumerate active physical/logical monitors using RandR 1.5.
    ///
    /// No guessed screen dimensions are returned when the extension is absent
    /// or too old; callers must handle the explicit unsupported error.
    pub fn monitors(&self) -> Result<Vec<Monitor>, X11Error> {
        let extension = self.connection.query_extension(b"RANDR")?.reply()?;
        if !extension.present {
            return Err(X11Error::RandrUnsupported);
        }

        let version = self.connection.randr_query_version(1, 5)?.reply()?;
        if version.major_version < 1 || (version.major_version == 1 && version.minor_version < 5) {
            return Err(X11Error::RandrUnsupported);
        }

        let reply = self
            .connection
            .randr_get_monitors(self.root_window(), true)?
            .reply()?;
        reply
            .monitors
            .into_iter()
            .map(|monitor| {
                let name = self.connection.get_atom_name(monitor.name)?.reply()?.name;
                Ok(Monitor {
                    name: String::from_utf8_lossy(&name).into_owned(),
                    x: i32::from(monitor.x),
                    y: i32::from(monitor.y),
                    width: u32::from(monitor.width),
                    height: u32::from(monitor.height),
                    primary: monitor.primary,
                })
            })
            .collect()
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

    /// Read a client's root-relative geometry without issuing any configure request.
    pub fn window_geometry(&self, window: WindowId) -> Result<Rect, X11Error> {
        let geometry = self.connection.get_geometry(window.0)?.reply()?;
        let translated = self
            .connection
            .translate_coordinates(window.0, self.root_window(), 0, 0)?
            .reply()?;
        Ok(Rect {
            x: i32::from(translated.dst_x),
            y: i32::from(translated.dst_y),
            width: u32::from(geometry.width),
            height: u32::from(geometry.height),
        })
    }

    pub fn window_desktop(&self, window: WindowId) -> Result<Option<u32>, X11Error> {
        let atom = Self::intern(&self.connection, b"_NET_WM_DESKTOP")?;
        let reply = self
            .connection
            .get_property(false, window.0, atom, AtomEnum::CARDINAL, 0, 1)?
            .reply()?;
        Ok(reply.value32().and_then(|mut values| values.next()))
    }

    /// Watch X11 events and call `reconcile` once after each relevant batch.
    ///
    /// The watcher blocks while idle, drains no more than 128 queued events per
    /// batch, and coalesces bursts into a single callback. The callback is
    /// invoked only for lifecycle/geometry changes or relevant EWMH properties.
    /// This method itself never mutates client geometry.
    pub fn watch_events(&self) -> Result<(), X11Error> {
        self.watch_events_with_reconcile(|| {})
    }

    pub fn watch_events_with_reconcile<F>(&self, mut reconcile: F) -> Result<(), X11Error>
    where
        F: FnMut(),
    {
        self.connection
            .change_window_attributes(
                self.root_window(),
                &ChangeWindowAttributesAux::new()
                    .event_mask(EventMask::SUBSTRUCTURE_NOTIFY | EventMask::PROPERTY_CHANGE),
            )?
            .check()?;
        if let Err(error) = self.connection.randr_select_input(
            self.root_window(),
            NotifyMask::SCREEN_CHANGE | NotifyMask::CRTC_CHANGE | NotifyMask::OUTPUT_CHANGE,
        ) {
            tracing::debug!(%error, "RandR monitor-change subscription unavailable");
        }
        for window in self.client_windows()? {
            self.subscribe_window_properties(window.0);
        }
        self.connection.flush()?;

        let relevant_properties: Vec<Atom> = [
            b"_NET_CLIENT_LIST".as_slice(),
            b"_NET_CURRENT_DESKTOP".as_slice(),
            b"_NET_NUMBER_OF_DESKTOPS".as_slice(),
            b"_NET_WORKAREA".as_slice(),
            b"_NET_WM_STATE".as_slice(),
            b"_NET_WM_DESKTOP".as_slice(),
            b"_NET_WM_WINDOW_TYPE".as_slice(),
            b"_NET_WM_NAME".as_slice(),
            b"WM_NAME".as_slice(),
            b"WM_CLASS".as_slice(),
        ]
        .iter()
        .map(|name| Self::intern(&self.connection, name))
        .collect::<Result<_, _>>()?;

        const MAX_EVENTS_PER_BATCH: usize = 128;
        loop {
            let first = self.connection.wait_for_event()?;
            let mut batch = Vec::with_capacity(MAX_EVENTS_PER_BATCH);
            batch.push(first);
            while batch.len() < MAX_EVENTS_PER_BATCH {
                match self.connection.poll_for_event()? {
                    Some(event) => batch.push(event),
                    None => break,
                }
            }

            let batch_len = batch.len();
            let mut reconcile_needed = false;
            for event in batch {
                match event {
                    Event::CreateNotify(event) => {
                        reconcile_needed = true;
                        self.subscribe_window_properties(event.window);
                        tracing::info!(window = event.window, "X11 window created");
                    }
                    Event::MapNotify(event) => {
                        reconcile_needed = true;
                        tracing::info!(window = event.window, "X11 window mapped");
                    }
                    Event::UnmapNotify(event) => {
                        reconcile_needed = true;
                        tracing::info!(window = event.window, "X11 window unmapped");
                    }
                    Event::ConfigureNotify(event) => {
                        reconcile_needed = true;
                        tracing::debug!(
                            window = event.window,
                            x = event.x,
                            y = event.y,
                            width = event.width,
                            height = event.height,
                            "X11 window geometry changed"
                        );
                    }
                    Event::DestroyNotify(event) => {
                        reconcile_needed = true;
                        tracing::info!(window = event.window, "X11 window destroyed");
                    }
                    Event::FocusIn(event) => {
                        reconcile_needed = true;
                        tracing::debug!(window = event.event, "X11 focus entered window");
                    }
                    Event::FocusOut(event) => {
                        reconcile_needed = true;
                        tracing::debug!(window = event.event, "X11 focus left window");
                    }
                    Event::RandrScreenChangeNotify(_) | Event::RandrNotify(_) => {
                        reconcile_needed = true;
                        tracing::info!("X11 monitor configuration changed");
                    }
                    Event::PropertyNotify(event) => {
                        if relevant_properties.contains(&event.atom) {
                            reconcile_needed = true;
                            tracing::debug!(
                                window = event.window,
                                atom = event.atom,
                                "relevant X11 property changed"
                            );
                        }
                    }
                    other => tracing::trace!(event = ?other, "unhandled X11 event"),
                }
            }
            tracing::debug!(
                events = batch_len,
                reconcile_needed,
                "processed coalesced X11 event batch"
            );
            if reconcile_needed {
                reconcile();
            }
        }
    }

    fn subscribe_window_properties(&self, window: Window) {
        let result = match self.connection.change_window_attributes(
            window,
            &ChangeWindowAttributesAux::new().event_mask(EventMask::PROPERTY_CHANGE),
        ) {
            Ok(cookie) => cookie.check().map_err(|error| error.to_string()),
            Err(error) => Err(error.to_string()),
        };
        if let Err(error) = result {
            // Clients may disappear between discovery/event delivery and subscription.
            tracing::debug!(window, %error, "could not subscribe to client property changes");
        }
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
        let type_atom = Self::intern(&self.connection, b"_NET_WM_WINDOW_TYPE")?;
        let type_reply = self
            .connection
            .get_property(false, window.0, type_atom, AtomEnum::ATOM, 0, 64)?
            .reply()?;
        let window_types: Vec<Atom> = type_reply
            .value32()
            .map(|values| values.collect())
            .unwrap_or_default();
        let has_type = |name: &[u8]| -> Result<bool, X11Error> {
            let atom = Self::intern(&self.connection, name)?;
            Ok(window_types.contains(&atom))
        };
        let transient_reply = self
            .connection
            .get_property(
                false,
                window.0,
                AtomEnum::WM_TRANSIENT_FOR,
                AtomEnum::WINDOW,
                0,
                1,
            )?
            .reply()?;
        let transient = transient_reply
            .value32()
            .and_then(|mut values| values.next())
            .is_some();

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
            dock: has_type(b"_NET_WM_WINDOW_TYPE_DOCK")?,
            desktop: has_type(b"_NET_WM_WINDOW_TYPE_DESKTOP")?,
            dialog: has_type(b"_NET_WM_WINDOW_TYPE_DIALOG")?,
            transient,
        })
    }
}
