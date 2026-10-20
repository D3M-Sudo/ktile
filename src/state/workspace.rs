use crate::layout::tree::Tree;
use crate::x11::monitor::WorkspaceArea;
use std::collections::HashMap;

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct WorkspaceId {
    pub desktop: u32,
    pub monitor: u32,
}

impl WorkspaceId {
    pub fn new(desktop: u32, monitor: u32) -> Self {
        Self { desktop, monitor }
    }
}

pub struct WorkspaceState {
    pub id: WorkspaceId,
    pub area: WorkspaceArea,
    pub tree: Tree,
}

impl WorkspaceState {
    pub fn new(id: WorkspaceId, area: WorkspaceArea) -> Self {
        Self {
            id,
            area,
            tree: Tree::new(),
        }
    }
}

#[derive(Default)]
pub struct WorkspaceManager {
    pub workspaces: HashMap<WorkspaceId, WorkspaceState>,
}

impl WorkspaceManager {
    pub fn new() -> Self {
        Self {
            workspaces: HashMap::new(),
        }
    }

    pub fn get_or_create(&mut self, id: WorkspaceId, area: WorkspaceArea) -> &mut WorkspaceState {
        self.workspaces
            .entry(id.clone())
            .or_insert_with(|| WorkspaceState::new(id, area))
    }
}
