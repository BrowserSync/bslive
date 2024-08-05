#[derive(Debug, thiserror::Error)]
pub enum YamlError {
    #[error(
        r#"
could not parse yaml file:
  {path}

input content was: (error at line: {line}, column: {column})

{input}

original error:

  {serde_error}
       "#
    )]
    ParseErrorWithLocation {
        path: String,
        input: String,
        serde_error: serde_yaml::Error,
        line: usize,
        column: usize,
    },
    #[error(
        r#"
could not parse yaml file:
  {path}

input content was:
{input}

original error:

  {serde_error}
       "#
    )]
    ParseError {
        path: String,
        input: String,
        serde_error: serde_yaml::Error,
    },
    #[error(
        r#"
could not parse raw yaml, input content was:
{input}

(error at line: {line}, column: {column})

original error:

  {serde_error}
       "#
    )]
    ParseRawInputErrorWithLocation {
        input: String,
        serde_error: serde_yaml::Error,
        line: usize,
        column: usize,
    },
    #[error(
        r#"
could not parse raw yaml, input content was:
{input}

original error:

  {serde_error}
       "#
    )]
    ParseRawInputError {
        input: String,
        serde_error: serde_yaml::Error,
    },
    #[error("Input file was empty: {path}")]
    EmptyError { path: String },
}
