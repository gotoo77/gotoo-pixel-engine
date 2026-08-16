use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StorageError {
    message: String,
}

impl StorageError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl fmt::Display for StorageError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for StorageError {}

pub trait LocalStorage {
    fn get(&mut self, key: &str) -> Result<Option<String>, StorageError>;
    fn set(&mut self, key: &str, value: &str) -> Result<(), StorageError>;
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct NoopStorage;

impl LocalStorage for NoopStorage {
    fn get(&mut self, _key: &str) -> Result<Option<String>, StorageError> {
        Ok(None)
    }

    fn set(&mut self, _key: &str, _value: &str) -> Result<(), StorageError> {
        Ok(())
    }
}

fn validate_storage_key(key: &str) -> Result<(), StorageError> {
    let portable = !key.is_empty()
        && key != "."
        && key != ".."
        && key.chars().all(|character| {
            character.is_ascii_alphanumeric() || matches!(character, '.' | '_' | '-')
        });

    if portable {
        Ok(())
    } else {
        Err(StorageError::new(
            "local storage keys may contain only ASCII letters, digits, '.', '_' and '-'",
        ))
    }
}

pub(crate) fn platform_storage() -> Box<dyn LocalStorage> {
    #[cfg(target_arch = "wasm32")]
    {
        Box::new(web::WebLocalStorage)
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        Box::new(native::FileLocalStorage::new())
    }
}

#[cfg(not(target_arch = "wasm32"))]
mod native {
    use std::fs;
    use std::io;
    use std::path::PathBuf;

    use directories::ProjectDirs;

    use super::{LocalStorage, StorageError, validate_storage_key};

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub(crate) struct FileLocalStorage {
        data_dir: Option<PathBuf>,
    }

    impl FileLocalStorage {
        pub(crate) fn new() -> Self {
            Self {
                data_dir: ProjectDirs::from("io.github", "gotoo77", "gotoo-pixel-engine")
                    .map(|dirs| dirs.data_local_dir().to_path_buf()),
            }
        }

        #[cfg(test)]
        fn new_at(data_dir: impl Into<PathBuf>) -> Self {
            Self {
                data_dir: Some(data_dir.into()),
            }
        }

        fn path_for_key(&self, key: &str) -> Result<PathBuf, StorageError> {
            let Some(data_dir) = self.data_dir.as_ref() else {
                return Err(StorageError::new("local data directory is unavailable"));
            };

            Ok(data_dir.join(storage_file_name(key)?))
        }
    }

    impl LocalStorage for FileLocalStorage {
        fn get(&mut self, key: &str) -> Result<Option<String>, StorageError> {
            let path = self.path_for_key(key)?;

            match fs::read_to_string(&path) {
                Ok(value) => Ok(Some(value)),
                Err(err) if err.kind() == io::ErrorKind::NotFound => Ok(None),
                Err(err) => Err(StorageError::new(format!(
                    "failed to read local storage key '{key}': {err}"
                ))),
            }
        }

        fn set(&mut self, key: &str, value: &str) -> Result<(), StorageError> {
            let path = self.path_for_key(key)?;
            let Some(parent) = path.parent() else {
                return Err(StorageError::new("local storage path has no parent"));
            };

            fs::create_dir_all(parent).map_err(|err| {
                StorageError::new(format!("failed to create local storage directory: {err}"))
            })?;
            fs::write(&path, value).map_err(|err| {
                StorageError::new(format!("failed to write local storage key '{key}': {err}"))
            })
        }
    }

    fn storage_file_name(key: &str) -> Result<String, StorageError> {
        validate_storage_key(key)?;
        Ok(format!("{key}.txt"))
    }

    #[cfg(test)]
    fn unique_test_dir(name: &str) -> PathBuf {
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system clock should be after unix epoch")
            .as_nanos();

        std::env::temp_dir().join(format!(
            "gotoo-pixel-engine-storage-{name}-{}-{nonce}",
            std::process::id()
        ))
    }

    #[cfg(test)]
    mod tests {
        use std::fs;

        use super::{FileLocalStorage, storage_file_name, unique_test_dir};
        use crate::LocalStorage;

        #[test]
        fn storage_file_name_accepts_only_portable_keys() {
            assert_eq!(
                storage_file_name("gotoo-pixel-engine.snake.best_score.v1"),
                Ok("gotoo-pixel-engine.snake.best_score.v1.txt".into())
            );
            assert!(storage_file_name("../bad/key").is_err());
            assert!(storage_file_name("a?b").is_err());
            assert!(storage_file_name("").is_err());
            assert!(storage_file_name(".").is_err());
            assert!(storage_file_name("..").is_err());
        }

        #[test]
        fn file_storage_returns_none_when_key_is_absent() {
            let dir = unique_test_dir("absent");
            let mut storage = FileLocalStorage::new_at(&dir);

            assert_eq!(storage.get("missing").expect("read should succeed"), None);

            let _ = fs::remove_dir_all(dir);
        }

        #[test]
        fn file_storage_persists_values_between_instances() {
            let dir = unique_test_dir("persist");
            let mut first = FileLocalStorage::new_at(&dir);
            first.set("snake.best", "37").expect("write should succeed");

            let mut second = FileLocalStorage::new_at(&dir);
            assert_eq!(
                second.get("snake.best").expect("read should succeed"),
                Some("37".into())
            );

            let _ = fs::remove_dir_all(dir);
        }

        #[test]
        fn file_storage_reports_read_errors() {
            let dir = unique_test_dir("read-error");
            fs::create_dir_all(&dir).expect("test dir should be created");
            fs::create_dir(dir.join("snake.best.txt")).expect("key path should be a directory");
            let mut storage = FileLocalStorage::new_at(&dir);

            assert!(storage.get("snake.best").is_err());

            let _ = fs::remove_dir_all(dir);
        }

        #[test]
        fn file_storage_reports_write_errors() {
            let dir = unique_test_dir("write-error");
            fs::write(&dir, "not a directory").expect("test path should be a file");
            let mut storage = FileLocalStorage::new_at(&dir);

            assert!(storage.set("snake.best", "37").is_err());

            let _ = fs::remove_file(dir);
        }
    }
}

#[cfg(target_arch = "wasm32")]
mod web {
    use wasm_bindgen::JsValue;

    use super::{LocalStorage, StorageError, validate_storage_key};

    #[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
    pub(crate) struct WebLocalStorage;

    impl LocalStorage for WebLocalStorage {
        fn get(&mut self, key: &str) -> Result<Option<String>, StorageError> {
            validate_storage_key(key)?;
            local_storage()?.get_item(key).map_err(|err| {
                StorageError::new(js_error_message("failed to read localStorage", err))
            })
        }

        fn set(&mut self, key: &str, value: &str) -> Result<(), StorageError> {
            validate_storage_key(key)?;
            local_storage()?.set_item(key, value).map_err(|err| {
                StorageError::new(js_error_message("failed to write localStorage", err))
            })
        }
    }

    fn local_storage() -> Result<web_sys::Storage, StorageError> {
        let window = web_sys::window().ok_or_else(|| StorageError::new("window is unavailable"))?;

        window
            .local_storage()
            .map_err(|err| StorageError::new(js_error_message("localStorage is unavailable", err)))?
            .ok_or_else(|| StorageError::new("localStorage is unavailable"))
    }

    fn js_error_message(context: &str, error: JsValue) -> String {
        match error.as_string() {
            Some(error) => format!("{context}: {error}"),
            None => context.into(),
        }
    }
}
