use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum ErrorCode {
    NotFound,
    Unauthorized,
    InternalError,
    ResourceNotFound,
    NotImplementedForOS,
    NotImplemented,
    InvalidInput,
    ResourceAlreadyExists,
}

impl ErrorCode {
    pub fn get_code_as_u16(&self) -> u16 {
        match self {
            ErrorCode::Unauthorized => 401,
            ErrorCode::NotFound => 404,
            ErrorCode::ResourceNotFound => 405,
            ErrorCode::NotImplementedForOS => 406,
            ErrorCode::NotImplemented => 407,
            ErrorCode::InvalidInput => 408,
            ErrorCode::ResourceAlreadyExists => 409,
            ErrorCode::InternalError => 500,
        }
    }

    #[allow(dead_code)]
    pub fn from_code(code: u16) -> Option<ErrorCode> {
        match code {
            401 => Some(ErrorCode::Unauthorized),
            404 => Some(ErrorCode::NotFound),
            405 => Some(ErrorCode::ResourceNotFound),
            406 => Some(ErrorCode::NotImplementedForOS),
            407 => Some(ErrorCode::NotImplemented),
            408 => Some(ErrorCode::InvalidInput),
            409 => Some(ErrorCode::ResourceAlreadyExists),
            500 => Some(ErrorCode::InternalError),
            _ => None,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Error {
    code: u16,
    message_from_code: ErrorCode,
    custom_message: String,
}
impl Error {
    pub fn new(code: ErrorCode, message: String) -> Self {
        Self {
            code: code.get_code_as_u16(),
            message_from_code: code,
            custom_message: message,
        }
    }
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap_or_else(|_| {
            r#"{"code":500,"message_from_code":"InternalError","custom_message":"Failed to serialize error"}"#.to_string()
        })
    }
}

//TODO a method which should be the constructor for the error code so pub fn new(code: u16, message: String) -> ErrorCode

//TODO a method which is called pub fn to _json(&self) -> String

//methode sollte dann so aussehen um den error aufzurufen
//Err(Error::new(ErrorCode::NotFound, "File not found".to_string()).to_json())

//oder
//Err(Error::new(ErrorCode::NotFound, format!("File not found: {}", file_path)).to_json())

//tests noch abändern
#[cfg(test)]
mod error_handling_tests {
    use crate::error_handling::{Error, ErrorCode};

    #[test]
    pub fn test() {
        let _x = Error::new(ErrorCode::NotFound, "File not found".to_string()).to_json();
        println!("Error: {:?}", _x);
    }
}
