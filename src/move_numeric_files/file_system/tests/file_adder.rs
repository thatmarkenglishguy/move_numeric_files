#[cfg(test)]
pub(crate) mod tests {
    use tempdir::TempDir;

    pub(crate) struct SubdirectoryFileAdder {
        sub_directory: std::path::PathBuf,
    }

    impl SubdirectoryFileAdder {
        pub(crate) fn new(sub_directory: std::path::PathBuf) -> Self {
            std::fs::create_dir(&sub_directory).unwrap();
            Self { sub_directory }
        }
    }

    pub(crate) trait FileAdderOperations {
        fn directory(&self) -> &std::path::Path;

        fn relative_file_path(&self, file_sub_path: &std::path::PathBuf) -> std::path::PathBuf {
            self.directory().join(file_sub_path)
        }

        fn with_file_paths<'a>(
            self,
            file_paths_iterator: impl Iterator<Item = std::path::PathBuf>,
        ) -> Self
        where
            Self: Sized
        {
            for file_sub_path in file_paths_iterator {
                let file_path = self.relative_file_path(&file_sub_path);
                std::fs::File::create(file_path).unwrap();
            }
            self
        }

        fn with_files<'a>(self, file_paths_iterator: impl Iterator<Item = &'a str>) -> Self
        where
            Self: Sized
        {
            let file_path_bufs =
                file_paths_iterator.map(|p| -> std::path::PathBuf { std::path::PathBuf::from(p) });
            self.with_file_paths(file_path_bufs)
        }

        fn with_files_slice(self, file_paths_slice: &[&str]) -> Self
        where
            Self: Sized
        {
            self.with_files(file_paths_slice.iter().cloned())
        }

        fn with_file_strings_slice(self, file_paths_slice: &[String]) -> Self
        where
            Self: Sized
        {
            self.with_files(file_paths_slice.iter().map(|string| string.as_str()))
        }

        fn with_sub_directory<A>(
            self,
            sub_directory: &str,
            sub_directory_file_adder_fn: impl Fn(SubdirectoryFileAdder) -> A,
        ) -> Self
        where
            Self: Sized
        {
            let sub_directory_path = self.relative_file_path(&std::path::PathBuf::from(sub_directory));
            let sub_directory_file_adder = SubdirectoryFileAdder::new(sub_directory_path);
            let _sub_directory_adder = sub_directory_file_adder_fn(sub_directory_file_adder);

            self
        }
    }

    pub(crate) struct TempFileAdder {
        directory: tempdir::TempDir,
    }

    impl TempFileAdder {
        const TEMPDIR_PREFIX: &'static str = "test_filesystem_walker";

        pub(crate) fn new() -> TempFileAdder {
            TempFileAdder {
                directory: TempDir::new(Self::TEMPDIR_PREFIX)
                    .expect(format!("Unable to create {}", Self::TEMPDIR_PREFIX).as_str()),
            }
        }
    }

    impl FileAdderOperations for TempFileAdder {
        fn directory(&self) -> &std::path::Path {
            self.directory.path()
        }
    }

    impl FileAdderOperations for SubdirectoryFileAdder {
        fn directory(&self) -> &std::path::Path {
            self.sub_directory.as_path()
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

    macro_rules! assert_file_does_not_exist {
        ($file_path:expr) => {
            assert!(
                ! $file_path.exists(),
                format!("file path does exist: {}", $file_path.to_str().unwrap())
            );
        };
    }

    #[test]
    fn test_deletes_temp_file_adder_directory() {
        let file_adder_directory: std::path::PathBuf;
        {
            let file_adder = TempFileAdder::new()
                .with_files_slice(&vec!["file_1", "file_2"])
                .with_sub_directory("sub01", |sub_adder| sub_adder.with_files_slice(&["sub01.txt"]))
                ;
            file_adder_directory = file_adder.directory().to_path_buf();
            assert_file_exists!(file_adder_directory);
        }

        assert_file_does_not_exist!(file_adder_directory);
    }


    #[test]
    fn test_create_test_files() {
        let test_files = vec!["file_1", "file_2"];

        let file_adder = TempFileAdder::new().with_files_slice(&test_files);

        for test_file in test_files {
            let file_path = file_adder.relative_file_path(&std::path::PathBuf::from(test_file));
            assert_file_exists!(file_path);
        }
    }

    #[test]
    fn test_create_test_files_in_a_subdirectory_using_closure_without_environment() {
        const SUBDIR_01: &str = "subdir01";
        const SUBDIR01_FILE01: &str = "file01.txt";
        const SUBDIR01_FILE02: &str = "file02.txt";

        let file_adder = TempFileAdder::new().with_sub_directory(SUBDIR_01, |subdir|
            subdir.with_files_slice(&vec![SUBDIR01_FILE01, SUBDIR01_FILE02])
        );

        for file_name in [SUBDIR01_FILE01, SUBDIR01_FILE02].iter() {
            let file_path = file_adder.relative_file_path(
                &[SUBDIR_01, file_name]
                    .iter()
                    .collect::<std::path::PathBuf>(),
            );
            assert_file_exists!(file_path);
        }
    }

    #[test]
    fn test_create_test_files_in_a_subdirectory_using_closure_with_environment() {
        const SUBDIR_01: &str = "subdir01";
        const SUBDIR01_FILE01: &str = "file01.txt";
        const SUBDIR01_FILE02: &str = "file02.txt";
        let subdirectory_files = vec![SUBDIR01_FILE01, SUBDIR01_FILE02];

        let file_adder = TempFileAdder::new().with_sub_directory(SUBDIR_01, |subdir|
            subdir.with_files_slice(&subdirectory_files)
        );

        for file_name in [SUBDIR01_FILE01, SUBDIR01_FILE02].iter() {
            let file_path = file_adder.relative_file_path(
                &[SUBDIR_01, file_name]
                    .iter()
                    .collect::<std::path::PathBuf>(),
            );
            assert_file_exists!(file_path);
        }
    }
}
