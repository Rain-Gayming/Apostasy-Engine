use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};

use crate::engine::assets::asset::*;
use crate::engine::assets::handle::*;

/// Type-erased asset storage for a single asset type
#[allow(unused)]
trait AssetStorage: Any + Send + Sync {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
    fn remove(&mut self, id: u64);
    fn contains(&self, id: u64) -> bool;
}

struct TypedStorage<T: Asset> {
    assets: HashMap<u64, T>,
    states: HashMap<u64, AssetLoadState>,
    path_to_id: HashMap<PathBuf, u64>,
    id_to_path: HashMap<u64, PathBuf>,
}

impl<T: Asset> TypedStorage<T> {
    fn new() -> Self {
        Self {
            assets: HashMap::new(),
            states: HashMap::new(),
            path_to_id: HashMap::new(),
            id_to_path: HashMap::new(),
        }
    }
}

impl<T: Asset> AssetStorage for TypedStorage<T> {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn remove(&mut self, id: u64) {
        self.assets.remove(&id);
        self.states.remove(&id);
        if let Some(path) = self.id_to_path.remove(&id) {
            self.path_to_id.remove(&path);
        }
    }

    fn contains(&self, id: u64) -> bool {
        self.assets.contains_key(&id)
    }
}

enum LoadOutcome {
    Success {
        id: u64,
        asset: Box<dyn Any + Send>,
        commit: fn(&mut AssetServerInner, u64, Box<dyn Any + Send>, &Path),
    },
    Failure {
        id: u64,
        message: String,
        mark_failed: fn(&mut AssetServerInner, u64, String),
    },
}

#[allow(unused)]
trait ErasedLoader: Send + Sync {
    fn extensions(&self) -> &[&str];
    fn load_erased(&self, path: &Path) -> LoadOutcome;
}

struct LoaderWrapper<L: AssetLoader> {
    loader: L,
}

impl<L: AssetLoader> ErasedLoader for LoaderWrapper<L> {
    fn extensions(&self) -> &[&str] {
        self.loader.extensions()
    }

    fn load_erased(&self, path: &Path) -> LoadOutcome {
        let id = Handle::<L::Asset>::new().id;

        match self.loader.load_sync(path) {
            Ok(asset) => LoadOutcome::Success {
                id,
                asset: Box::new(asset),
                commit: |inner, id, boxed, path| {
                    let asset = *boxed
                        .downcast::<L::Asset>()
                        .expect("asset type mismatch in commit");

                    inner
                        .storage
                        .entry(TypeId::of::<L::Asset>())
                        .or_insert_with(|| Box::new(TypedStorage::<L::Asset>::new()));

                    let storage = inner
                        .storage
                        .get_mut(&TypeId::of::<L::Asset>())
                        .unwrap()
                        .as_any_mut()
                        .downcast_mut::<TypedStorage<L::Asset>>()
                        .unwrap();

                    storage.assets.insert(id, asset);
                    storage.states.insert(id, AssetLoadState::Loaded);
                    storage.path_to_id.insert(path.to_path_buf(), id);
                    storage.id_to_path.insert(id, path.to_path_buf());
                },
            },
            Err(e) => LoadOutcome::Failure {
                id,
                message: e.to_string(),
                mark_failed: |inner, id, msg| {
                    inner
                        .storage
                        .entry(TypeId::of::<L::Asset>())
                        .or_insert_with(|| Box::new(TypedStorage::<L::Asset>::new()));

                    let storage = inner
                        .storage
                        .get_mut(&TypeId::of::<L::Asset>())
                        .unwrap()
                        .as_any_mut()
                        .downcast_mut::<TypedStorage<L::Asset>>()
                        .unwrap();

                    storage.states.insert(id, AssetLoadState::Failed(msg));
                },
            },
        }
    }
}

pub(crate) struct AssetServerInner {
    storage: HashMap<TypeId, Box<dyn AssetStorage>>,
    ext_to_loader: HashMap<String, usize>,
    loaders: Vec<Arc<dyn ErasedLoader>>,
    root: PathBuf,
}

impl AssetServerInner {
    fn new(root: impl Into<PathBuf>) -> Self {
        Self {
            storage: HashMap::new(),
            ext_to_loader: HashMap::new(),
            loaders: Vec::new(),
            root: root.into(),
        }
    }

    fn typed_storage<T: Asset>(&self) -> Option<&TypedStorage<T>> {
        self.storage
            .get(&TypeId::of::<T>())?
            .as_any()
            .downcast_ref::<TypedStorage<T>>()
    }

    fn typed_storage_mut<T: Asset>(&mut self) -> Option<&mut TypedStorage<T>> {
        self.storage
            .get_mut(&TypeId::of::<T>())?
            .as_any_mut()
            .downcast_mut::<TypedStorage<T>>()
    }

    fn ensure_storage<T: Asset>(&mut self) {
        self.storage
            .entry(TypeId::of::<T>())
            .or_insert_with(|| Box::new(TypedStorage::<T>::new()));
    }
}

#[derive(Clone)]
pub struct AssetServer {
    inner: Arc<RwLock<AssetServerInner>>,
}

impl AssetServer {
    //// create a new asset server rooted at the given path
    pub fn new(asset_dir: impl Into<PathBuf>) -> Self {
        let inner = Arc::new(RwLock::new(AssetServerInner::new(asset_dir)));
        AssetServer { inner }
    }

    /// Register an asset loader
    pub fn register_loader<L: AssetLoader>(&mut self, loader: L) {
        let mut inner = self.inner.write().unwrap();
        let index = inner.loaders.len();
        let extensions: Vec<String> = loader.extensions().iter().map(|s| s.to_string()).collect();
        inner.loaders.push(Arc::new(LoaderWrapper { loader }));
        for ext in extensions {
            inner.ext_to_loader.insert(ext, index);
        }
    }

    pub fn reload_all<T: Asset>(&self) -> Vec<(PathBuf, Result<(), AssetLoadError>)> {
        // Collect all (id, path) pairs first to release the read lock before loading.
        let entries: Vec<(u64, PathBuf)> = {
            let inner = self.inner.read().unwrap();
            inner
                .typed_storage::<T>()
                .map(|s| {
                    s.id_to_path
                        .iter()
                        .map(|(&id, path)| (id, path.clone()))
                        .collect()
                })
                .unwrap_or_default()
        };

        let mut results = Vec::with_capacity(entries.len());

        for (id, full_path) in entries {
            // Find the right loader by extension.
            let loader_arc = {
                let inner = self.inner.read().unwrap();
                let ext = full_path
                    .extension()
                    .and_then(|e| e.to_str())
                    .map(|e| e.to_lowercase());

                let arc = ext
                    .as_deref()
                    .and_then(|e| inner.ext_to_loader.get(e))
                    .map(|&idx| Arc::clone(&inner.loaders[idx]));

                arc
            };

            let Some(loader_arc) = loader_arc else {
                results.push((
                    full_path,
                    Err(AssetLoadError::other("no loader for extension")),
                ));
                continue;
            };

            // Run the loader outside of any lock.
            let outcome = loader_arc.load_erased(&full_path);

            let result = {
                let mut inner = self.inner.write().unwrap();
                match outcome {
                    LoadOutcome::Success { asset, commit, .. } => {
                        // commit() inserts under a *new* id — we want the original id.
                        // Downcast manually and replace in-place.
                        //
                        // We re-use the `commit` closure only to get the asset out of
                        // the box in a type-safe way; we then fix up the id mapping.
                        let new_id = {
                            // Temporarily commit under the new id, then swap.
                            let new_id = {
                                // peek at what id commit() would use
                                inner
                                    .storage
                                    .entry(TypeId::of::<T>())
                                    .or_insert_with(|| Box::new(TypedStorage::<T>::new()));
                                let storage = inner
                                    .storage
                                    .get_mut(&TypeId::of::<T>())
                                    .unwrap()
                                    .as_any_mut()
                                    .downcast_mut::<TypedStorage<T>>()
                                    .unwrap();

                                // Downcast the box directly so we never touch a foreign id.
                                if let Ok(typed) = asset.downcast::<T>() {
                                    storage.assets.insert(id, *typed);
                                    storage.states.insert(id, AssetLoadState::Loaded);
                                    Some(id)
                                } else {
                                    None
                                }
                            };
                            new_id
                        };

                        if new_id.is_some() {
                            Ok(())
                        } else {
                            Err(AssetLoadError::other("asset type mismatch during reload"))
                        }
                    }
                    LoadOutcome::Failure { message, .. } => {
                        let mut inner_storage = inner
                            .storage
                            .get_mut(&TypeId::of::<T>())
                            .and_then(|s| s.as_any_mut().downcast_mut::<TypedStorage<T>>());
                        if let Some(storage) = inner_storage {
                            storage
                                .states
                                .insert(id, AssetLoadState::Failed(message.clone()));
                        }
                        Err(AssetLoadError::other(message))
                    }
                }
            };

            results.push((full_path, result));
        }

        results
    }

    pub fn reload<T: Asset>(&self, handle: Handle<T>) -> Result<(), AssetLoadError> {
        let full_path = self
            .path_of(handle)
            .ok_or_else(|| AssetLoadError::other("handle has no associated path"))?;

        let loader_arc = {
            let inner = self.inner.read().unwrap();
            let ext = full_path
                .extension()
                .and_then(|e| e.to_str())
                .map(|e| e.to_lowercase())
                .ok_or_else(|| AssetLoadError::other("path has no extension"))?;

            let idx = *inner
                .ext_to_loader
                .get(&ext)
                .ok_or_else(|| AssetLoadError::NoLoader { extension: ext })?;

            Arc::clone(&inner.loaders[idx])
        };

        let outcome = loader_arc.load_erased(&full_path);

        let mut inner = self.inner.write().unwrap();
        match outcome {
            LoadOutcome::Success { asset, .. } => {
                inner
                    .storage
                    .entry(TypeId::of::<T>())
                    .or_insert_with(|| Box::new(TypedStorage::<T>::new()));

                let storage = inner
                    .storage
                    .get_mut(&TypeId::of::<T>())
                    .unwrap()
                    .as_any_mut()
                    .downcast_mut::<TypedStorage<T>>()
                    .unwrap();

                let typed = asset
                    .downcast::<T>()
                    .map_err(|_| AssetLoadError::other("asset type mismatch during reload"))?;

                storage.assets.insert(handle.id, *typed);
                storage.states.insert(handle.id, AssetLoadState::Loaded);
                Ok(())
            }
            LoadOutcome::Failure { message, .. } => {
                if let Some(storage) = inner
                    .storage
                    .get_mut(&TypeId::of::<T>())
                    .and_then(|s| s.as_any_mut().downcast_mut::<TypedStorage<T>>())
                {
                    storage
                        .states
                        .insert(handle.id, AssetLoadState::Failed(message.clone()));
                }
                Err(AssetLoadError::other(message))
            }
        }
    }

    pub fn load<T: Asset>(&self, path: impl AsRef<Path>) -> Result<Handle<T>, AssetLoadError> {
        let (full_path, loader_arc) = {
            let inner = self.inner.read().unwrap();
            let full_path = inner.root.join(path.as_ref());
            let ext = full_path
                .extension()
                .and_then(|e| e.to_str())
                .map(|e| e.to_lowercase())
                .ok_or_else(|| AssetLoadError::other("Path has no extension"))?;

            let idx = *inner
                .ext_to_loader
                .get(&ext)
                .ok_or_else(|| AssetLoadError::NoLoader {
                    extension: ext.clone(),
                })?;

            (full_path, Arc::clone(&inner.loaders[idx]))
        };

        let outcome = loader_arc.load_erased(&full_path);

        let id = {
            let mut inner = self.inner.write().unwrap();
            match outcome {
                LoadOutcome::Success { id, asset, commit } => {
                    commit(&mut inner, id, asset, &full_path);
                    id
                }
                LoadOutcome::Failure {
                    id,
                    message,
                    mark_failed,
                } => {
                    println!("marking failed");
                    println!("{}", message);
                    mark_failed(&mut inner, id, message.clone());
                    return Err(AssetLoadError::other(message));
                }
            }
        };
        Ok(Handle::with_id(id))
    }

    pub fn load_cached<T: Asset>(
        &self,
        path: impl AsRef<Path>,
    ) -> Result<Handle<T>, AssetLoadError> {
        let full_path = {
            let inner = self.inner.read().unwrap();
            inner.root.join(path.as_ref())
        };

        // Check cache first
        {
            let inner = self.inner.read().unwrap();
            if let Some(storage) = inner.typed_storage::<T>() {
                if let Some(&id) = storage.path_to_id.get(&full_path) {
                    return Ok(Handle::with_id(id));
                }
            }
        }

        // Not cached — do a fresh load
        self.load(path)
    }

    pub fn insert<T: Asset>(&self, asset: T) -> Handle<T> {
        let handle = Handle::<T>::new();
        let mut inner = self.inner.write().unwrap();
        inner.ensure_storage::<T>();
        let storage = inner.typed_storage_mut::<T>().unwrap();
        storage.assets.insert(handle.id, asset);
        storage.states.insert(handle.id, AssetLoadState::Loaded);
        handle
    }

    /// Insert an asset at a virtual path so it can be found by `load_cached`.
    pub fn insert_at<T: Asset>(&self, asset: T, path: impl Into<PathBuf>) -> Handle<T> {
        let handle = self.insert(asset);
        let path = path.into();
        let mut inner = self.inner.write().unwrap();
        if let Some(storage) = inner.typed_storage_mut::<T>() {
            storage.path_to_id.insert(path.clone(), handle.id);
            storage.id_to_path.insert(handle.id, path);
        }
        handle
    }

    pub fn get<T: Asset>(
        &self,
        handle: Handle<T>,
    ) -> Option<impl std::ops::Deref<Target = T> + '_> {
        let inner = self.inner.read().unwrap();
        let _ = inner.typed_storage::<T>()?.assets.get(&handle.id)?;
        // Return a guard-backed reference
        Some(AssetRef {
            _guard: self.inner.read().unwrap(),
            id: handle.id,
            _marker: std::marker::PhantomData::<T>,
        })
    }

    /// Get a mutable reference to the asset behind a handle.
    pub fn get_mut<T: Asset>(
        &self,
        handle: Handle<T>,
    ) -> Option<impl std::ops::DerefMut<Target = T> + '_> {
        {
            let inner = self.inner.write().unwrap();

            let exists = inner
                .typed_storage::<T>()
                .map(|s| s.assets.contains_key(&handle.id))
                .unwrap_or(false);
            if !exists {
                return None;
            }
        }

        Some(AssetMutRef {
            _guard: self.inner.write().unwrap(),
            id: handle.id,
            _marker: std::marker::PhantomData::<T>,
        })
    }

    pub fn get_cloned<T: Asset + Clone>(&self, handle: Handle<T>) -> Option<T> {
        let inner = self.inner.read().unwrap();
        inner.typed_storage::<T>()?.assets.get(&handle.id).cloned()
    }

    pub fn load_state<T: Asset>(&self, handle: Handle<T>) -> AssetLoadState {
        let inner = self.inner.read().unwrap();
        inner
            .typed_storage::<T>()
            .and_then(|s| s.states.get(&handle.id).cloned())
            .unwrap_or(AssetLoadState::Unloaded)
    }

    /// Returns `true` if the asset is fully loaded and ready to use.
    pub fn is_loaded<T: Asset>(&self, handle: Handle<T>) -> bool {
        self.load_state(handle).is_loaded()
    }

    /// Remove an asset from the server, freeing its memory.
    /// Any existing `Handle<T>` pointing to it will return `None` on `get()`.
    pub fn remove<T: Asset>(&self, handle: Handle<T>) {
        let mut inner = self.inner.write().unwrap();
        if let Some(storage) = inner.typed_storage_mut::<T>() {
            storage.remove(handle.id);
        }
    }

    /// Look up the file path an asset was loaded from, if any.
    pub fn path_of<T: Asset>(&self, handle: Handle<T>) -> Option<PathBuf> {
        let inner = self.inner.read().unwrap();
        inner
            .typed_storage::<T>()?
            .id_to_path
            .get(&handle.id)
            .cloned()
    }

    /// Returns the total number of loaded assets of type `T`.
    pub fn count<T: Asset>(&self) -> usize {
        let inner = self.inner.read().unwrap();
        inner
            .typed_storage::<T>()
            .map(|s| s.assets.len())
            .unwrap_or(0)
    }
}

struct AssetRef<'a, T: Asset> {
    _guard: std::sync::RwLockReadGuard<'a, AssetServerInner>,
    id: u64,
    _marker: std::marker::PhantomData<T>,
}

impl<'a, T: Asset> std::ops::Deref for AssetRef<'a, T> {
    type Target = T;
    fn deref(&self) -> &T {
        self._guard
            .typed_storage::<T>()
            .unwrap()
            .assets
            .get(&self.id)
            .unwrap()
    }
}

struct AssetMutRef<'a, T: Asset> {
    _guard: std::sync::RwLockWriteGuard<'a, AssetServerInner>,
    id: u64,
    _marker: std::marker::PhantomData<T>,
}

impl<'a, T: Asset> std::ops::Deref for AssetMutRef<'a, T> {
    type Target = T;
    fn deref(&self) -> &T {
        self._guard
            .typed_storage::<T>()
            .unwrap()
            .assets
            .get(&self.id)
            .unwrap()
    }
}

impl<'a, T: Asset> std::ops::DerefMut for AssetMutRef<'a, T> {
    fn deref_mut(&mut self) -> &mut T {
        self._guard
            .typed_storage_mut::<T>()
            .unwrap()
            .assets
            .get_mut(&self.id)
            .unwrap()
    }
}
