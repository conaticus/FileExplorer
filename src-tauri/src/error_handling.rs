use serde::{Deserialize, Serialize};


/*
//Hier gehört noch was hin XD
pub enum ErrorCode {
    NotFound,
    Unauthorized,
    InternalError,
}
 */

/*
impl ErrorCode {
    pub fn code(&self) -> u16 {
        match self {
            ErrorCode::NotFound => 404,
            ErrorCode::Unauthorized => 401,
            ErrorCode::InternalError => 500,
        }
    }

    pub fn message(&self) -> &'static str {
        match self {
            ErrorCode::NotFound => "Not Found",
            ErrorCode::Unauthorized => "Unauthorized",
            ErrorCode::InternalError => "Internal Server Error",
        }
    }

    pub fn from_code(code: u16) -> Option<ErrorCode> {
        match code {
            404 => Some(ErrorCode::NotFound),
            401 => Some(ErrorCode::Unauthorized),
            500 => Some(ErrorCode::InternalError),
            _ => None,
        }
    }

}

//Hier gehört noch was hin XD
pub struct Error {
    code: ErrorCode,
    message_from_code: String,
    custom_message: String,
}
impl Error {
    pub fn new(code: ErrorCode, message: String) -> Self {
       let message_from_code = code.message();
        Self {
            code,
            message_from_code: message_from_code.to_string(),
            custom_message: message,
        }
    }
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap()
    }
}

    //TODO a method which should be the constructor for the error code so pub fn new(code: u16, message: String) -> ErrorCode 
    
    //TODO a method which is called pub fn to _json(&self) -> String 
    
    //methode sollte dann so aussehen um den error aufzurufen 
    Err(Error::new(ErrorCode::NotFound, "File not found".to_string()).to_json())   
    
    //oder
    Err(Error::new(ErrorCode::NotFound, format!("File not found: {}", file_path)).to_json())   

 */

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
        //& string ist problematisch weil die referenz zu dem string nicht mehr existiert wenn die methode beendet ist
        let error = Error::new(ErrorCodes::NotFound, "File not found");
        let json_error = error.to_json();
        let _x = ErrorCodes::NotFound as u16;
        println!("Error in JSON format: {}", json_error);
        
    }
    
}