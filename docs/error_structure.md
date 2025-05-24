# Error Structure Documentation

## Content

- Error Code
- Error Code Structure

---

## Error Code

| Error Code | Error Message         |
|------------|-----------------------|
| 401        | Unauthorized          |
| 404        | NotFound              |
| 405        | ResourceNotFound      |
| 406        | NotImplementedForOS   |
| 407        | NotImplemented        |
| 408        | InvalidInput          |
| 409        | ResourceAlreadyExists |
| 500        | InternalError         |

---


## Error Code Structure

### Output Structure

```json
{
  "error_code": 401,
  "error_message": "Unauthorized",
  "custom_message": "Custom Message"
}
```

### Input Structure map_err

```
fs::create_dir_all(parent).map_err(|e| {
    log_error!(format!("Failed to create parent directory: {}", e).as_str());
    Error::new(
        ErrorCode::InternalError,
        format!(
            "Failed to create parent directory '{}': {}",
            parent.display(),
            e
        ),
    )
    .to_json()
})?;
```

### Input Structure return Err

```
return Err(Error::new(
    ErrorCode::InvalidInput,
    "Destination path exists but is not a directory".to_string(),
)
.to_json());
```