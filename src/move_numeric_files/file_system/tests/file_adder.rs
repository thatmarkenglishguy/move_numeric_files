#[cfg(test)]
pub(crate) mod tests {
    use tempdir::TempDir;

    pub(crate) struct FileAdder {
        directory: tempdir::TempDir,
    }

    impl FileAdder {
        const TEMPDIR_PREFIX: &'static str = "test_filesystem_walker";

        fn relative_file_path(&self, file_sub_path: &std::path::PathBuf) -> std::path::PathBuf {
            self.directory.path().join(file_sub_path)
        }

        pub(crate) fn new() -> FileAdder {
            FileAdder {
                directory: TempDir::new(Self::TEMPDIR_PREFIX)
                    .expect(format!("Unable to create {}", Self::TEMPDIR_PREFIX).as_str()),
            }
        }

        /// Add files to temp directory
        fn with_file_paths<'a>(
            self,
            file_paths_iterator: impl Iterator<Item = std::path::PathBuf>,
        ) -> FileAdder {
            for file_sub_path in file_paths_iterator {
                let file_path = self.relative_file_path(&file_sub_path);
                std::fs::File::create(file_path).unwrap();
            }
            self
        }

        pub(crate) fn with_files<'a>(
            self,
            file_paths_iterator: impl Iterator<Item = &'a str>,
        ) -> Self {
            let file_path_bufs =
                file_paths_iterator.map(|p| -> std::path::PathBuf { std::path::PathBuf::from(p) });
            self.with_file_paths(file_path_bufs)
        }

        pub(crate) fn with_files_slice(self, file_paths_slice: &[&str]) -> Self {
            self.with_files(file_paths_slice.iter().cloned())
        }

        pub(crate) fn directory(&self) -> &std::path::Path {
            self.directory.path()
        }
    }

    macro_rules! assert_file_exists {
        ($file_path:expr) => {
            assert!(
                $file_path.exists(),
                format!("file path does not exist: {}", $file_path.to_str().unwrap())
            );
        };
    }

    #[test]
    fn test_create_test_files() {
        let test_files = vec!["file_1", "file_2"];

        let file_adder = FileAdder::new().with_files_slice(&test_files);

        for test_file in test_files {
            let file_path = file_adder.relative_file_path(&std::path::PathBuf::from(test_file));
            assert_file_exists!(file_path);
        }
    }
}
