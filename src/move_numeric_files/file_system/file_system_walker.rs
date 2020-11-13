use std::path::Path;

struct FileWalker<'a> {
    root: &'a std::path::Path,
}

trait Walker {
    fn visit(&mut self, path: &std::path::Path, directories: &[String], files: &[String]);
}

impl<F> Walker for F
where
    F: Fn(&std::path::Path, &[String], &[String]),
{
    fn visit(&mut self, path: &Path, directories: &[String], files: &[String]) {
        self(path, directories, files);
    }
}

impl<'a> FileWalker<'a> {
    fn new(root: &std::path::Path) -> FileWalker {
        FileWalker { root }
    }

    pub(crate) fn walk<W: Walker>(&self, walker: &mut W) -> std::io::Result<()> {
        let mut directory_paths: Vec<std::path::PathBuf> = vec![];

        {
            let mut directories: Vec<String> = Vec::new();
            let mut files: Vec<String> = Vec::new();

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

            walker.visit(self.root, &directories, &files);
        }

        for directory_path in &directory_paths {
            FileWalker::new(directory_path).walk(walker)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::move_numeric_files::file_system::file_system_walker::{FileWalker, Walker};
    use crate::move_numeric_files::file_system::tests::file_adder::tests::FileAdder;
    use std::cell::RefCell;
    use std::path::Path;

    #[test]
    fn walk_test_files_with_closure() {
        let test_files = vec!["first", "second"];
        let file_adder = FileAdder::new().with_files_slice(&test_files);
        let file_walker = FileWalker::new(file_adder.directory());

        // BOOOOOOO
        let seen_directories: RefCell<Vec<String>> = RefCell::new(vec![]);
        let seen_files: RefCell<Vec<String>> = RefCell::new(vec![]);
        let walk_executed = RefCell::new(false);

        file_walker
            .walk(
                &mut |parent: &std::path::Path, directories: &[String], files: &[String]| {
                    (*seen_directories.borrow_mut()).extend_from_slice(directories);
                    (*seen_files.borrow_mut()).extend_from_slice(files);
                    (*walk_executed.borrow_mut()) = true;
                    assert_eq!(directories.len(), 0);
                    assert_eq!(parent, file_adder.directory());
                },
            )
            .unwrap();

        assert!(*walk_executed.borrow());
        assert_eq!((*seen_directories.borrow()).len(), 0);
        assert_eq!(*seen_files.borrow(), test_files);
    }

    /// Same as using closure but refactored into struct...
    struct TestWalker {
        seen_directories: Vec<String>,
        seen_files: Vec<String>,
        walk_executed: bool,
    }

    impl TestWalker {
        fn new() -> TestWalker {
            TestWalker {
                seen_directories: vec![],
                seen_files: vec![],
                walk_executed: false,
            }
        }

        fn verify(&self, test_directories: &[&str], test_files: &[&str]) {
            assert!(self.walk_executed);
            assert_eq!(self.seen_directories, test_directories);
            assert_eq!(self.seen_files, test_files);
        }
    }

    impl Walker for TestWalker {
        fn visit(&mut self, path: &Path, directories: &[String], files: &[String]) {
            self.walk_executed = true;
            self.seen_directories.extend_from_slice(directories);
            self.seen_files.extend_from_slice(files);
        }
    }

    #[test]
    fn walk_test_files_with_trait_implementation() {
        let expected_directories: Vec<&str> = vec![];
        let test_files = vec!["first", "second"];
        let file_adder = FileAdder::new().with_files_slice(&test_files);
        let file_walker = FileWalker::new(file_adder.directory());
        let mut test_walker = TestWalker::new();

        file_walker.walk(&mut test_walker).unwrap();

        test_walker.verify(&expected_directories, &test_files);
    }
}
