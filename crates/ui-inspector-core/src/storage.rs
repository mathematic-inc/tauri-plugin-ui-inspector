//! Atomic, bounded, project-local reference persistence.

use std::{
    fs::{self, File, OpenOptions},
    io::{BufReader, BufWriter, Write},
    path::{Path, PathBuf},
};

use fs4::FileExt;
use image::{ImageFormat, RgbaImage};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::{ElementReference, ReferenceId, error::StorageError};

const REFERENCE_FILE: &str = "reference.json";
const WINDOW_FILE: &str = "window.png";
const ELEMENT_FILE: &str = "element.png";
const LOCK_FILE: &str = ".lock";

/// Reference storage settings.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct StorageConfig {
    /// Project-local root directory, usually `.ui-inspector`.
    #[ts(type = "string")]
    pub root: PathBuf,
    /// Maximum persisted references retained after each save.
    pub max_history: usize,
}

impl StorageConfig {
    /// Creates settings rooted at `root` with a 100-reference history.
    ///
    /// # Examples
    ///
    /// ```
    /// use tauri_ui_inspector_core::StorageConfig;
    ///
    /// let config = StorageConfig::new(".ui-inspector");
    /// assert_eq!(config.max_history, 100);
    /// ```
    #[must_use]
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self {
            root: root.into(),
            max_history: 100,
        }
    }
}

/// A reference and its directory on disk.
#[derive(Clone, Debug)]
pub struct ReferenceEntry {
    reference: ElementReference,
    directory: PathBuf,
}

impl ReferenceEntry {
    /// Returns the deserialized reference.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let entry = storage.last()?.unwrap();
    /// assert!(entry.reference().id.starts_with("ui_"));
    /// ```
    #[inline]
    #[must_use]
    pub fn reference(&self) -> &ElementReference {
        &self.reference
    }

    /// Returns the reference directory.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let entry = storage.last()?.unwrap();
    /// assert!(entry.directory().ends_with(&entry.reference().id));
    /// ```
    #[inline]
    #[must_use]
    pub fn directory(&self) -> &Path {
        &self.directory
    }

    /// Consumes the entry and returns its reference.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let reference = storage.last()?.unwrap().into_reference();
    /// assert!(reference.id.starts_with("ui_"));
    /// ```
    #[inline]
    #[must_use]
    pub fn into_reference(self) -> ElementReference {
        self.reference
    }
}

/// Project-local reference store.
#[derive(Clone, Debug)]
pub struct Storage {
    config: StorageConfig,
}

impl Storage {
    /// Creates a store from `config`.
    ///
    /// # Examples
    ///
    /// ```
    /// use tauri_ui_inspector_core::{Storage, StorageConfig};
    ///
    /// let storage = Storage::new(StorageConfig::new(".ui-inspector"));
    /// assert!(storage.root().ends_with(".ui-inspector"));
    /// ```
    #[must_use]
    pub fn new(config: StorageConfig) -> Self {
        Self { config }
    }

    /// Returns the storage root.
    ///
    /// # Examples
    ///
    /// ```
    /// use tauri_ui_inspector_core::{Storage, StorageConfig};
    ///
    /// let storage = Storage::new(StorageConfig::new(".ui-inspector"));
    /// assert_eq!(storage.root().to_string_lossy(), ".ui-inspector");
    /// ```
    #[inline]
    #[must_use]
    pub fn root(&self) -> &Path {
        &self.config.root
    }

    /// Returns the directory containing persisted references.
    ///
    /// # Examples
    ///
    /// ```
    /// use tauri_ui_inspector_core::{Storage, StorageConfig};
    ///
    /// let storage = Storage::new(StorageConfig::new(".ui-inspector"));
    /// assert!(storage.refs_dir().ends_with("refs"));
    /// ```
    #[must_use]
    pub fn refs_dir(&self) -> PathBuf {
        self.config.root.join("refs")
    }

    /// Persists a reference and optional screenshots atomically.
    ///
    /// # Errors
    ///
    /// Returns [`StorageError`] when directories, JSON, PNGs, locking, or the
    /// final atomic rename fail.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let directory = storage.save(&reference, Some(&window), Some(&element))?;
    /// assert!(directory.join("reference.json").exists());
    /// ```
    pub fn save(
        &self,
        reference: &ElementReference,
        window: Option<&RgbaImage>,
        element: Option<&RgbaImage>,
    ) -> Result<PathBuf, StorageError> {
        let refs_dir = self.refs_dir();
        fs::create_dir_all(&refs_dir).map_err(|error| StorageError::io(&refs_dir, error))?;
        let _lock = StorageLock::acquire(&self.config.root.join(LOCK_FILE))?;
        let final_dir = refs_dir.join(&reference.id);
        if final_dir.exists() {
            return Err(StorageError::io(
                &final_dir,
                std::io::Error::new(
                    std::io::ErrorKind::AlreadyExists,
                    "reference directory already exists",
                ),
            ));
        }

        let temporary =
            tempfile::tempdir_in(&refs_dir).map_err(|error| StorageError::io(&refs_dir, error))?;
        write_json(&temporary.path().join(REFERENCE_FILE), reference)?;
        if let Some(image) = window {
            write_png(&temporary.path().join(WINDOW_FILE), image)?;
        }
        if let Some(image) = element {
            write_png(&temporary.path().join(ELEMENT_FILE), image)?;
        }
        let temporary_path = temporary.keep();
        fs::rename(&temporary_path, &final_dir)
            .map_err(|error| StorageError::io(&final_dir, error))?;
        self.cleanup_locked()?;
        Ok(final_dir)
    }

    /// Reads a reference by identifier.
    ///
    /// # Errors
    ///
    /// Returns [`StorageError::NotFound`] when the reference directory is
    /// absent, or another [`StorageError`] when JSON or filesystem access
    /// fails.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let entry = storage.get("@ui_01ARZ3NDEKTSV4RRFFQ69G5FAV")?;
    /// assert_eq!(entry.reference().id, "ui_01ARZ3NDEKTSV4RRFFQ69G5FAV");
    /// ```
    pub fn get(&self, id: impl AsRef<str>) -> Result<ReferenceEntry, StorageError> {
        let id = ReferenceId::parse(id.as_ref()).map_err(|_| StorageError::NotFound {
            id: id.as_ref().to_owned(),
        })?;
        let directory = self.refs_dir().join(id.as_str());
        if !directory.is_dir() {
            return Err(StorageError::NotFound { id: id.to_string() });
        }
        let reference = read_json(&directory.join(REFERENCE_FILE))?;
        Ok(ReferenceEntry {
            reference,
            directory,
        })
    }

    /// Lists references newest first.
    ///
    /// # Errors
    ///
    /// Returns [`StorageError`] when the reference directory or JSON files
    /// cannot be read.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let entries = storage.list()?;
    /// assert!(entries.windows(2).all(|pair| pair[0].reference().id > pair[1].reference().id));
    /// ```
    pub fn list(&self) -> Result<Vec<ReferenceEntry>, StorageError> {
        let refs_dir = self.refs_dir();
        if !refs_dir.exists() {
            return Ok(Vec::new());
        }
        let mut ids = reference_directories(&refs_dir)?;
        ids.sort_unstable_by(|left, right| right.cmp(left));
        ids.into_iter().map(|id| self.get(id)).collect()
    }

    /// Returns the newest reference.
    ///
    /// # Errors
    ///
    /// Returns [`StorageError`] when listing or reading references fails.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// if let Some(entry) = storage.last()? {
    ///     assert!(entry.reference().id.starts_with("ui_"));
    /// }
    /// ```
    pub fn last(&self) -> Result<Option<ReferenceEntry>, StorageError> {
        self.list().map(|mut entries| entries.drain(..).next())
    }

    /// Deletes one reference directory.
    ///
    /// # Errors
    ///
    /// Returns [`StorageError::NotFound`] when the reference is absent and
    /// [`StorageError`] when deletion fails.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// storage.delete("ui_01ARZ3NDEKTSV4RRFFQ69G5FAV")?;
    /// ```
    pub fn delete(&self, id: impl AsRef<str>) -> Result<(), StorageError> {
        let entry = self.get(id)?;
        let _lock = StorageLock::acquire(&self.config.root.join(LOCK_FILE))?;
        fs::remove_dir_all(entry.directory())
            .map_err(|error| StorageError::io(entry.directory(), error))
    }

    /// Deletes every persisted reference.
    ///
    /// # Errors
    ///
    /// Returns [`StorageError`] when a reference directory cannot be removed.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// storage.clear()?;
    /// assert!(storage.list()?.is_empty());
    /// ```
    pub fn clear(&self) -> Result<(), StorageError> {
        let refs_dir = self.refs_dir();
        let _lock = StorageLock::acquire(&self.config.root.join(LOCK_FILE))?;
        if refs_dir.exists() {
            fs::remove_dir_all(&refs_dir).map_err(|error| StorageError::io(&refs_dir, error))?;
        }
        Ok(())
    }

    fn cleanup_locked(&self) -> Result<(), StorageError> {
        if self.config.max_history == 0 {
            return Ok(());
        }
        let refs_dir = self.refs_dir();
        let mut ids = reference_directories(&refs_dir)?;
        ids.sort_unstable();
        let remove_count = ids.len().saturating_sub(self.config.max_history);
        for id in ids.into_iter().take(remove_count) {
            let path = refs_dir.join(id);
            fs::remove_dir_all(&path).map_err(|error| StorageError::io(&path, error))?;
        }
        Ok(())
    }
}

struct StorageLock(File);

impl StorageLock {
    fn acquire(path: &Path) -> Result<Self, StorageError> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|error| StorageError::io(parent, error))?;
        }
        let file = OpenOptions::new()
            .create(true)
            .read(true)
            .write(true)
            .truncate(false)
            .open(path)
            .map_err(|error| StorageError::io(path, error))?;
        FileExt::lock(&file).map_err(|error| StorageError::io(path, error))?;
        Ok(Self(file))
    }
}

impl std::fmt::Debug for StorageLock {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("StorageLock")
    }
}

impl Drop for StorageLock {
    fn drop(&mut self) {
        let _ = FileExt::unlock(&self.0);
    }
}

fn reference_directories(refs_dir: &Path) -> Result<Vec<String>, StorageError> {
    let entries = fs::read_dir(refs_dir).map_err(|error| StorageError::io(refs_dir, error))?;
    let mut ids = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|error| StorageError::io(refs_dir, error))?;
        let file_type = entry
            .file_type()
            .map_err(|error| StorageError::io(entry.path(), error))?;
        if file_type.is_dir() {
            let name = entry.file_name().to_string_lossy().into_owned();
            if ReferenceId::parse(&name).is_ok() {
                ids.push(name);
            }
        }
    }
    Ok(ids)
}

fn read_json(path: &Path) -> Result<ElementReference, StorageError> {
    let file = File::open(path).map_err(|error| StorageError::io(path, error))?;
    serde_json::from_reader(BufReader::new(file)).map_err(StorageError::from)
}

fn write_json(path: &Path, reference: &ElementReference) -> Result<(), StorageError> {
    let file = File::create(path).map_err(|error| StorageError::io(path, error))?;
    let mut writer = BufWriter::new(file);
    serde_json::to_writer_pretty(&mut writer, reference)?;
    writer
        .write_all(b"\n")
        .map_err(|error| StorageError::io(path, error))?;
    writer
        .flush()
        .map_err(|error| StorageError::io(path, error))?;
    Ok(())
}

fn write_png(path: &Path, image: &RgbaImage) -> Result<(), StorageError> {
    image.save_with_format(path, ImageFormat::Png)?;
    Ok(())
}
