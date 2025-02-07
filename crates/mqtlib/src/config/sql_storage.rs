use url::Url;
use validator::{Validate, ValidationError};

#[derive(Clone, Debug, Default, Validate)]
pub struct SqlStorage {
    #[validate(length(min = 1), custom(function = "validate_connection_string"))]
    pub connection_string: String,
}

impl SqlStorage {
    pub fn scheme(&self) -> String {
        let url = Url::parse(self.connection_string.as_ref()).unwrap();
        url.scheme().to_string()
    }
}

fn validate_connection_string(connection_string: &str) -> Result<(), ValidationError> {
    let url = Url::parse(connection_string)
        .map_err(|_| ValidationError::new("Connection string is not a valid URL"))?;

    match url.scheme() {
        "sqlite" => Ok(()),
        _ => Err(ValidationError::new(
            "Only scheme sqlite is currently supported",
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_sqlite_in_memory() {
        let conf = SqlStorage {
            connection_string: "sqlite::memory:".to_string(),
        };
        let result = conf.validate();

        assert!(result.is_ok());
    }

    #[test]
    fn validate_sqlite_temporary_file() {
        let conf = SqlStorage {
            connection_string: "sqlite://".to_string(),
        };
        let result = conf.validate();

        assert!(result.is_ok());
    }

    #[test]
    fn validate_sqlite_file_no_authority() {
        let conf = SqlStorage {
            connection_string: "sqlite:data.db".to_string(),
        };
        let result = conf.validate();

        assert!(result.is_ok());
    }

    #[test]
    fn validate_sqlite_file_with_authority() {
        let conf = SqlStorage {
            connection_string: "sqlite://data.db".to_string(),
        };
        let result = conf.validate();

        assert!(result.is_ok());
    }

    #[test]
    fn validate_invalid_file() {
        let conf = SqlStorage {
            connection_string: "file.db".to_string(),
        };
        let result = conf.validate();

        assert!(result.is_err());
    }
}
