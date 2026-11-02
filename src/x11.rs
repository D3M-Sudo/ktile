//! X11 adapter. Geometry writes are intentionally not enabled in this foundation
//! milestone; the backend must validate EWMH state before any future mutation.

use thiserror::Error;
use x11rb::{connection::Connection, protocol::xproto::ConnectionExt, rust_connection::RustConnection};

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
        let atoms = connection.intern_atom(false, b"_NET_SUPPORTING_WM_CHECK")?.reply()?;
        let property = connection.get_property(false, root, atoms.atom, x11rb::protocol::xproto::AtomEnum::WINDOW, 0, 1)?.reply()?;
        if property.value32().and_then(|mut values| values.next()).is_none() {
            return Err(X11Error::NoWindowManager);
        }
        Ok(Self { connection, screen_num })
    }

    pub fn screen_size(&self) -> (u16, u16) {
        let screen = &self.connection.setup().roots[self.screen_num];
        (screen.width_in_pixels, screen.height_in_pixels)
    }

    pub fn root_window(&self) -> u32 {
        self.connection.setup().roots[self.screen_num].root
    }
}
