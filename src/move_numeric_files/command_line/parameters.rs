use argh::FromArgs;
use std::ops::Deref;

fn pwd() -> std::path::PathBuf {
    std::env::current_dir().expect("Unable to get current working directory")
}

#[derive(FromArgs, PartialEq, Debug)]
/// Move files prefixed with a numeric identifier "up" so that there are no duplicate numbers.
pub struct Parameters {
    #[argh(option, default = "1")]
    /// the first number to start checking for duplicate numbers at.
    pub start: u32,

    #[argh(option, default = "pwd()", long = "directory")]
    /// directory to search for files in.
    directory_buf: std::path::PathBuf,

    #[argh(option)]
    /// when a class is found, the name of the file to keep.
    /// If not specified, the first file name encountered is kept.
    pub keep_file: Option<String>,
}

impl Parameters {
    fn from_command_line() -> Parameters {
        argh::from_env()
    }

    fn directory(self: &Self) -> &std::path::Path {
        self.directory_buf.deref()
    }
}

#[cfg(test)]
mod tests {
    use crate::move_numeric_files::command_line::parameters::Parameters;
    use argh::FromArgs;

    #[test]
    fn omitting_keep_file_works() -> Result<(), std::io::Error> {
        let parameters = Parameters::from_args(&["move_numeric_files_test"], &["--start", "3"])
            .expect("failed parameters parse just start");
        assert_eq!(
            parameters,
            Parameters {
                start: 3,
                directory_buf: std::env::current_dir()?,
                keep_file: None
            }
        );
        Ok(())
    }

    #[test]
    fn specifying_keep_file_works() -> Result<(), std::io::Error> {
        let keep_file_name = "keep_file_test_name";
        let parameters = Parameters::from_args(
            &["move_numeric_files_test"],
            &["--start", "3", "--keep-file", keep_file_name],
        )
        .expect("failed parameters parse with keep file");
        assert_eq!(
            parameters,
            Parameters {
                start: 3,
                directory_buf: std::env::current_dir()?,
                keep_file: Some(keep_file_name.to_string())
            }
        );
        Ok(())
    }

    #[test]
    fn specifying_directory_works() {
        let starting_directory = "/starting_directory";
        let parameters = Parameters::from_args(
            &["move_numeric_files_test"],
            &["--start", "3", "--directory", starting_directory],
        )
        .expect("failed parameters parse just with directory");
        assert_eq!(
            parameters,
            Parameters {
                start: 3,
                directory_buf: std::path::PathBuf::from(starting_directory),
                keep_file: None,
            }
        );
    }
}
