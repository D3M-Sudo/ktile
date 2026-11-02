use serde::{Deserialize, Serialize};

/// Stable identifier for a window in the backend currently managing it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct WindowId(pub u32);

/// Window states that affect whether K-tile may manage geometry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct WindowState {
    pub fullscreen: bool,
    pub maximized_horizontal: bool,
    pub maximized_vertical: bool,
    pub modal: bool,
    pub above: bool,
    pub below: bool,
    pub skip_taskbar: bool,
    pub skip_pager: bool,
    pub sticky: bool,
    pub minimized: bool,
    pub dock: bool,
    pub desktop: bool,
    pub dialog: bool,
    pub transient: bool,
}

impl WindowState {
    /// Conservative policy: do not resize special or user-managed windows.
    pub fn is_tile_eligible(self) -> bool {
        !(self.fullscreen
            || self.maximized_horizontal
            || self.maximized_vertical
            || self.modal
            || self.above
            || self.below
            || self.skip_taskbar
            || self.skip_pager
            || self.minimized
            || self.dock
            || self.desktop
            || self.dialog
            || self.transient)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WindowInfo {
    pub id: WindowId,
    pub title: Option<String>,
    pub app_class: Option<String>,
    pub workspace: u32,
    pub monitor: u32,
    pub state: WindowState,
    pub floating: bool,
}

impl WindowInfo {
    pub fn is_tile_eligible(&self) -> bool {
        !self.floating && self.state.is_tile_eligible()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ordinary_window_is_eligible() {
        assert!(WindowState::default().is_tile_eligible());
    }

    #[test]
    fn sticky_windows_remain_eligible_for_current_workspace_planning() {
        assert!(
            WindowState {
                sticky: true,
                ..Default::default()
            }
            .is_tile_eligible()
        );
    }

    #[test]
    fn special_window_states_are_excluded() {
        let states = [
            WindowState {
                fullscreen: true,
                ..Default::default()
            },
            WindowState {
                maximized_horizontal: true,
                ..Default::default()
            },
            WindowState {
                maximized_vertical: true,
                ..Default::default()
            },
            WindowState {
                modal: true,
                ..Default::default()
            },
            WindowState {
                above: true,
                ..Default::default()
            },
            WindowState {
                below: true,
                ..Default::default()
            },
            WindowState {
                skip_taskbar: true,
                ..Default::default()
            },
            WindowState {
                skip_pager: true,
                ..Default::default()
            },
            WindowState {
                minimized: true,
                ..Default::default()
            },
            WindowState {
                dock: true,
                ..Default::default()
            },
            WindowState {
                desktop: true,
                ..Default::default()
            },
            WindowState {
                dialog: true,
                ..Default::default()
            },
            WindowState {
                transient: true,
                ..Default::default()
            },
        ];
        assert!(states.into_iter().all(|state| !state.is_tile_eligible()));
    }
}
