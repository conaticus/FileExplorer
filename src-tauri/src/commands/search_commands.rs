use crate::error_handling::{Error, ErrorCode};

//TODO
#[tauri::command]
pub async fn _search_command(snipped: String) -> Result<String, String> {
    // Simulate a search operation
    if snipped.is_empty() {
        return Err(Error::new(
            ErrorCode::InvalidInput,
            "Search term cannot be empty".to_string(),
        )
            .to_json());
    }
    
    // Here you would typically perform the search operation
    // For demonstration, we will just return a success message
    Ok(format!("Search results for: {}", snipped))
}

#[cfg(test)]
mod tests_search_commands {
    use super::*;
    
    #[tokio::test]
    async fn _search_command_test() {
        let result = _search_command("test".to_string()).await;
        assert!(result.is_ok());
        let search_result = result.unwrap();
        assert_eq!(search_result, "Search results for: test");
    }
    
    #[tokio::test]
    async fn failed_to_search_command_because_of_search_term_is_empty_test() {
        let result = _search_command("".to_string()).await;

        assert!(
            result.is_err(),
            "Failed test (should throw an error): {:?}",
            result
        );
        assert!(
            result
                .clone()
                .unwrap_err()
                .contains("Search term cannot be empty"),
            "Error message does not match expected value"
        );

        assert!(
            result.clone().unwrap_err().contains("408"),
            "Error message does not match expected value"
        );

        assert!(
            result.unwrap_err().contains("InvalidInput"),
            "Error message does not match expected value"
        );
    }
}