use serde::{
    Deserialize,
    Deserializer
};

use std::{
    fmt::{
        Display,
        Formatter,
        Result as FmtResult
    }
};

#[derive(Debug, Clone, Serialize)]
crate enum DashboardPermissionLevel {
    Admin,
    Editor,
    Viewer
}

impl Display for DashboardPermissionLevel {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Self::Admin => write!(f, "admin"),
            Self::Editor => write!(f, "editor"),
            Self::Viewer => write!(f, "viewer")
        }
    }
}
