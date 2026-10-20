use crate::errors::KtileError;
use x11rb::connection::Connection;
use x11rb::protocol::xproto::{AtomEnum, ConnectionExt, Window};

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct WindowClass {
    pub instance: String,
    pub class_name: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SizeHints {
    pub min_width: Option<u32>,
    pub min_height: Option<u32>,
    pub max_width: Option<u32>,
    pub max_height: Option<u32>,
}

/// Reads ICCCM `WM_CLASS` property in read-only mode.
pub fn read_wm_class<C: Connection>(
    conn: &C,
    window: Window,
) -> Result<Option<WindowClass>, KtileError> {
    let reply =
        match conn.get_property(false, window, AtomEnum::WM_CLASS, AtomEnum::STRING, 0, 1024) {
            Ok(cookie) => match cookie.reply() {
                Ok(reply) => reply,
                Err(_) => return Ok(None),
            },
            Err(_) => return Ok(None),
        };

    if reply.type_ == AtomEnum::STRING.into() && !reply.value.is_empty() {
        let parts: Vec<&[u8]> = reply.value.split(|&b| b == 0).collect();
        let instance = parts
            .first()
            .map(|b| String::from_utf8_lossy(b).to_string())
            .unwrap_or_default();
        let class_name = parts
            .get(1)
            .map(|b| String::from_utf8_lossy(b).to_string())
            .unwrap_or_default();

        if !instance.is_empty() || !class_name.is_empty() {
            return Ok(Some(WindowClass {
                instance,
                class_name,
            }));
        }
    }

    Ok(None)
}

/// Reads ICCCM `WM_TRANSIENT_FOR` property in read-only mode.
pub fn read_wm_transient_for<C: Connection>(
    conn: &C,
    window: Window,
) -> Result<Option<Window>, KtileError> {
    let reply = match conn.get_property(
        false,
        window,
        AtomEnum::WM_TRANSIENT_FOR,
        AtomEnum::WINDOW,
        0,
        1,
    ) {
        Ok(cookie) => match cookie.reply() {
            Ok(reply) => reply,
            Err(_) => return Ok(None),
        },
        Err(_) => return Ok(None),
    };

    if reply.type_ == AtomEnum::WINDOW.into() && reply.value.len() >= 4 {
        let target = u32::from_ne_bytes(reply.value[0..4].try_into().unwrap());
        if target != 0 {
            return Ok(Some(target));
        }
    }

    Ok(None)
}

/// Reads ICCCM `WM_NORMAL_HINTS` property in read-only mode without modifying it.
/// NOTE: According to ADR-11 and Cortile analysis, WM_NORMAL_HINTS must NEVER be written/rewritten.
pub fn read_wm_normal_hints<C: Connection>(
    conn: &C,
    window: Window,
) -> Result<Option<SizeHints>, KtileError> {
    let reply = match conn.get_property(
        false,
        window,
        AtomEnum::WM_SIZE_HINTS,
        AtomEnum::WM_SIZE_HINTS,
        0,
        18,
    ) {
        Ok(cookie) => match cookie.reply() {
            Ok(reply) => reply,
            Err(_) => return Ok(None),
        },
        Err(_) => return Ok(None),
    };

    if reply.type_ == AtomEnum::WM_SIZE_HINTS.into() && reply.value.len() >= 36 {
        let flags = u32::from_ne_bytes(reply.value[0..4].try_into().unwrap());
        let mut hints = SizeHints::default();

        // PMinSize flag = 1 << 4 (16)
        if (flags & (1 << 4)) != 0 && reply.value.len() >= 28 {
            let min_w = u32::from_ne_bytes(reply.value[20..24].try_into().unwrap());
            let min_h = u32::from_ne_bytes(reply.value[24..28].try_into().unwrap());
            hints.min_width = Some(min_w);
            hints.min_height = Some(min_h);
        }

        // PMaxSize flag = 1 << 5 (32)
        if (flags & (1 << 5)) != 0 && reply.value.len() >= 36 {
            let max_w = u32::from_ne_bytes(reply.value[28..32].try_into().unwrap());
            let max_h = u32::from_ne_bytes(reply.value[32..36].try_into().unwrap());
            hints.max_width = Some(max_w);
            hints.max_height = Some(max_h);
        }

        return Ok(Some(hints));
    }

    Ok(None)
}
