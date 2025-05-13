use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
enum ErrorCodes {
    NotFound = 404,
    PermissionDenied = 403,
    Unknown = 500,
}



#[derive(Debug, Serialize, Deserialize)]
struct Error {
    code: ErrorCodes,
    message: String,
}

impl Error {
    fn new(code: ErrorCodes, message: &str) -> Self {
        
        Self {
            code,
            message: message.to_string(),
        }
    }

    fn to_json(&self) -> String {
        serde_json::to_string(&self).unwrap()
    }
}


// chatgpt fragen nach custom contructor durch fn new,
// automatisch im error ein neues Feld, welches automatisch mit der Zahl initialisiert wird


#[cfg(test)]
mod error_handling_tests {
    use crate::error_handling::{Error, ErrorCodes};

    #[test]
    pub fn test() {
        let error = Error::new(ErrorCodes::NotFound, "File not found");
        let json_error = error.to_json();
        let _x = ErrorCodes::NotFound as u16;
        println!("Error in JSON format: {}", json_error);
    }
    
}