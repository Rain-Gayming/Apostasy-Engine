use std::any::Any;
use std::path::Path;

/// The loaded state of an asset in the AssetServer
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AssetLoadState {
    /// The asset is not loaded
    Unloaded,
    /// The asset is loaded and ready to be used
    Loaded,
    /// The asset is currently loading
    Loading,
    /// Asset failed to load
    Failed(String),
}

impl AssetLoadState {
    /// Returns true if the asset is loaded
    pub fn is_loaded(&self) -> bool {
        matches!(self, AssetLoadState::Loaded)
    }

    /// Returns true if the asset has failed to load
    pub fn is_failed(&self) -> bool {
        matches!(self, AssetLoadState::Failed(_))
    }
}

/// Core trait for all assets
pub trait Asset: Any + Send + Sync + 'static {
    /// Human-readable name for this asset type
    fn asset_type_name() -> &'static str
    where
        Self: Sized;
}

pub trait AssetLoader: Send + Sync + 'static {
    /// The asset type this loader can load
    type Asset: Asset;

    /// The file extension this loader can load
    fn extensions(&self) -> &[&str];

    /// Synchronously load an asset from the given path
    fn load_sync(&self, path: &Path) -> Result<Self::Asset, AssetLoadError>;
}

/// Errors that can happen when loading an asset
#[derive(Debug, thiserror::Error)]
pub enum AssetLoadError {
    #[error("IO error loading asset at '{path}': {source}")]
    Io {
        path: String,
        #[source]
        source: std::io::Error,
    },
    #[error("Parse error loading asset at '{path}': {message}")]
    Parse { path: String, message: String },

    #[error("No loader registered for extension '{extension}'")]
    NoLoader { extension: String },

    #[error("Asset not found at path: {0}")]
    NotFound(String),

    #[error("Asset load error: {0}")]
    Other(String),
}

impl AssetLoadError {
    pub fn other(msg: impl Into<String>) -> Self {
        Self::Other(msg.into())
    }
}
