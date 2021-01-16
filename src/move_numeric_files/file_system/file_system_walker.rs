struct FileWalker<'a> {
    root: &'a std::path::Path,
}

trait Walker<I> {
    fn visit(&mut self, root: &std::path::Path, directories: I, files: I);
}

impl FileWalker<'_> {
    fn new(root: &std::path::Path) -> FileWalker {
        FileWalker { root }
    }

    pub(crate) fn walk<W>(&self, walker: &mut W) -> std::io::Result<()>
    where
        W: Walker<std::vec::IntoIter<String>>,
    {
        let mut directory_paths = vec![];

        {
            let mut directories = Vec::new();
            let mut files = Vec::new();

            for entry in std::fs::read_dir(self.root)? {
                let entry = entry?;
                let entry_path = entry.path();
                let path_string =
                    String::from(entry.path().file_name().and_then(|s| s.to_str()).unwrap());
                if entry_path.is_dir() {
                    let entry_path_clone = entry_path.clone();
                    directory_paths.push(entry_path_clone);
                    directories.push(path_string);
                } else {
                    files.push(path_string);
                }
            }

            walker.visit(self.root, directories.into_iter(), files.into_iter());
        }

        for directory_path in directory_paths.into_iter() {
            let file_walker = FileWalker::new(&directory_path);
            file_walker.walk(walker)?;
        }

        Ok(())
    }
}

impl<I, F> Walker<I> for F
where
    F: FnMut(&std::path::Path, I, I),
{
    fn visit(&mut self, root: &std::path::Path, directories: I, files: I) {
        self(root, directories, files);
    }
}

#[cfg(test)]
mod tests {
    use crate::move_numeric_files::file_system::file_system_walker::{FileWalker, Walker};
    //use crate::move_numeric_files::file_system::tests::file_adder::tests::FileAdder;
    use crate::move_numeric_files::file_system::tests::file_adder::tests::{
        FileAdderOperations, TempFileAdder,
    };
    use std::cell::RefCell;

    fn slice_to_vector<S, V>(slice: &[S], transmuter: impl Fn(S) -> V) -> Vec<V>
        where S: Clone
    {
        slice
            .iter()
            .cloned()
            .map(transmuter)
            .collect::<Vec<_>>()
    }

    fn str_slice_to_string_vector(slice: &[&str]) -> Vec<String> {
        slice_to_vector(slice, String::from)
    }

    #[test]
    fn walk_test_files_with_closure() {
        let test_files = vec!["first", "second"]
            .iter()
            .cloned()
            .map(String::from)
            .collect::<Vec<_>>();
        let file_adder = TempFileAdder::new().with_file_strings_slice(&test_files);
        let file_walker = FileWalker::new(file_adder.directory());

        // BOOOOOOO
        let seen_path = RefCell::new(vec![]);
        let seen_directories = RefCell::new(vec![]);
        let seen_files = RefCell::new(vec![]);
        let walk_executed = RefCell::new(false);

        file_walker
            .walk(&mut |path: &std::path::Path,
                        directories: std::vec::IntoIter<_>,
                        files: std::vec::IntoIter<_>| {
                assert_eq!(
                    directories.len(),
                    0,
                    "Expected 0 directories. Got [{}]",
                    directories.clone().collect::<Vec<_>>().join(", ")
                );
                (*seen_path.borrow_mut()).push(path.to_path_buf());
                (*seen_directories.borrow_mut()).extend(directories);
                (*seen_files.borrow_mut()).extend(files);
                (*walk_executed.borrow_mut()) = true;
            })
            .unwrap();

        assert!(*walk_executed.borrow());
        assert_eq!((*seen_path.borrow()), vec!(file_adder.directory()));
        assert_eq!(
            (*seen_directories.borrow()).len(),
            0,
            "Expected 0 seen directories. Got [{}]",
            (*seen_directories.borrow()).join(", ")
        );
        assert_eq!(*seen_files.borrow(), test_files);
    }

    /// Same as using closure but refactored into struct...
    struct TestWalker {
        seen_paths: Vec<std::path::PathBuf>,
        seen_directories: Vec<String>,
        seen_files: Vec<String>,
        walk_executed: bool,
    }

    impl TestWalker {
        fn new() -> TestWalker {
            TestWalker {
                seen_paths: vec![],
                seen_directories: vec![],
                seen_files: vec![],
                walk_executed: false,
            }
        }

        fn verify<'p>(
            &mut self,
            test_paths: impl IntoIterator<Item = &'p std::path::Path>,
            test_directories: impl Iterator<Item = String>,
            test_files: impl Iterator<Item = String>,
        ) {
            let expected_directories = test_directories.collect::<Vec<_>>();
            let expected_files = test_files.collect::<Vec<_>>();
            let expected_paths = test_paths
                .into_iter()
                .map(|path| path.to_path_buf())
                .collect::<Vec<_>>();

            self.seen_files.sort();
            self.seen_directories.sort();
            self.seen_paths.sort();
            assert!(self.walk_executed);
            assert_eq!(self.seen_paths, expected_paths);
            assert_eq!(self.seen_directories, expected_directories);
            assert_eq!(self.seen_files, expected_files);
        }
    }

    impl<I> Walker<I> for TestWalker
    where
        I: Iterator<Item = String>,
    {
        fn visit(&mut self, _path: &std::path::Path, directories: I, files: I) {
            self.walk_executed = true;
            self.seen_paths.push(_path.to_path_buf());
            self.seen_directories.extend(directories);
            let relative_file_path = files
                .map(|file_name| _path.join(file_name))
                .map(|path| path.to_str().map(String::from).unwrap())
            ;
            self.seen_files.extend(relative_file_path);
        }
    }

    #[test]
    fn walk_test_files_with_trait_implementation() {
        let expected_directories: Vec<_> = vec![];
        let test_files = str_slice_to_string_vector(&["first", "second"]);
        let file_adder = TempFileAdder::new().with_file_strings_slice(&test_files);
        let expected_files: Vec<_> = test_files
            .iter()
            .map(|subfilename| file_adder.relative_file_path(&std::path::PathBuf::from(subfilename)).to_str().map(String::from).unwrap())
            .collect::<Vec<_>>()
            ;
        let expected_paths = vec![file_adder.directory()];
        let file_walker = FileWalker::new(file_adder.directory());
        let mut test_walker = TestWalker::new();

        file_walker.walk(&mut test_walker).unwrap();

        test_walker.verify(
            expected_paths.into_iter(),
            expected_directories.into_iter(),
            expected_files.into_iter(),
        );
    }

    #[test]
    fn walk_test_sub_directory_files_with_trait_implementation() {
        const SUBDIR_01: &str = "subdir01";

        let expected_directories: Vec<_> = vec![String::from(SUBDIR_01)];
        let test_files = str_slice_to_string_vector(&["first", "second"]);
        let sub_test_files = str_slice_to_string_vector(&["first_sub", "second_sub"]);
        let file_adder = TempFileAdder::new()
            .with_file_strings_slice(&test_files)
            .with_sub_directory(SUBDIR_01,
                                |subdir01_adder|
                                    subdir01_adder.with_file_strings_slice(&sub_test_files)
            );
        let subdir01_path = file_adder.relative_file_path(&std::path::PathBuf::from(SUBDIR_01));
        let mut expected_files: Vec<_> = test_files
            .iter()
            .map(|subfilename| file_adder.relative_file_path(&std::path::PathBuf::from(subfilename)).to_str().map(String::from).unwrap())
            .collect::<Vec<_>>()
            ;
        expected_files.extend(sub_test_files.iter().cloned().map(
            |subfilename| subdir01_path.join(subfilename).to_str().map(String::from).unwrap()
        ));
        let expected_paths = vec![file_adder.directory(), subdir01_path.as_path()];
        let file_walker = FileWalker::new(file_adder.directory());
        let mut test_walker = TestWalker::new();

        file_walker.walk(&mut test_walker).unwrap();

        test_walker.verify(
            expected_paths.into_iter(),
            expected_directories.into_iter(),
            expected_files.into_iter(),
        );
    }
}
