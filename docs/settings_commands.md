# Tauri Settings Commands Documentation

Error Structure as json can be found [here](./error_structure.md).

## Content
- [Get All Settings](#get_settings_as_json-endpoint)
- [Get a Specific Setting](#get_setting_field-endpoint)
- [Update a Setting Field](#update_settings_field-endpoint)
- [Update Multiple Settings](#update_multiple_settings_command-endpoint)
- [Reset Settings](#reset_settings-endpoint)

# Current settings
The current settings consist of the following fields. A nearer explanation of each field can be found text them or unter the settings object.

```json
{
   "darkmode":false,
   "custom_themes":[
      
   ],
   "default_theme":"",
   "default_themes_path":"",
   "default_folder_path_on_opening":"",
   "default_view":"Grid",
   "font_size":"Medium",
   "show_hidden_files_and_folders":false,
   "show_details_panel":false,
   "accent_color":"#000000",
   "confirm_delete":true,
   "auto_refresh_dir":true,
   "sort_direction":"Acscending",
   "sort_by":"Name",
   "double_click":"OpenFilesAndFolders",
   "show_file_extensions":true,
   "terminal_height":240,
   "enable_animations_and_transitions":true,
   "enable_virtual_scroll_for_large_directories":false,
   "abs_file_path_buf":"/tmp/.tmpBX63JY",
   "enable_suggestions":true,
   "highlight_matches":true,
   "backend_settings":{
      "search_engine_config":{
         "search_engine_enabled":true,
         "max_results":20,
         "preferred_extensions":[
            "txt",
            "pdf",
            "docx",
            "xlsx",
            "md",
            "rs",
            "js",
            "html",
            "css",
            "json",
            "png",
            "jpg"
         ],
         "excluded_patterns":[
            ".git",
            "node_modules",
            "target"
         ],
         "cache_size":1000,
         "ranking_config":{
            "frequency_weight":0.05,
            "max_frequency_boost":0.5,
            "recency_weight":1.5,
            "recency_lambda":0.000011574074,
            "context_same_dir_boost":0.4,
            "context_parent_dir_boost":0.2,
            "extension_boost":2.0,
            "extension_query_boost":0.25,
            "exact_match_boost":1.0,
            "prefix_match_boost":0.3,
            "contains_match_boost":0.1,
            "directory_ranking_boost":0.2
         },
         "prefer_directories":false,
         "cache_ttl":{
            "secs":300,
            "nanos":0
         },
         "collect_usage_stats":true,
         "indexing_logging_enabled":false,
         "search_logging_enabled":false,
         "search_timeout_ms":5000,
         "result_score_threshold":0.1,
         "min_query_length":null,
         "max_indexed_files":null,
         "max_index_depth":null,
         "index_hidden_files":false,
         "follow_symlinks":false,
         "fuzzy_trigram_threshold":0.5,
         "fuzzy_search_enabled":true,
         "case_sensitive_search":false,
         "group_results_by_directory":true,
         "persistent_index_path":null,
         "index_compression_enabled":true,
         "indexing_priority":1,
         "default_search_operator":"AND",
         "enable_wildcard_search":false,
         "indexing_batch_size":100,
         "retry_failed_indexing":true
      },
      "logging_config":{
         "logging_level":"Full",
         "json_log":false
      },
      "default_checksum_hash":"SHA256"
   }
}
```

# `get_settings_as_json` endpoint

---
## Parameters
- None

## Returns
- String: A JSON string representation of all current settings.

## Example call
```typescript jsx
useEffect(() => {
    const fetchSettings = async () => {
        try {
            const settingsJson = await invoke("get_settings_as_json");
            const settings = JSON.parse(settingsJson);
            console.log("Current settings:", settings);
        } catch (error) {
            console.error("Error fetching settings:", error);
        }
    };

    fetchSettings();
}, []);
```

# `get_setting_field` endpoint

---
## Parameters
- `key`: A string representing the setting key to retrieve.

## Returns
- Ok(Value): The value of the requested setting if found.
- Err(String): An error message if the setting key doesn't exist or another error occurred.

## Example call
```typescript jsx
useEffect(() => {
    const fetchThemeSetting = async () => {
        try {
            const themeValue = await invoke("get_setting_field", { key: "theme" });
            console.log("Theme setting:", themeValue);
        } catch (error) {
            console.error("Error fetching theme setting:", error);
        }
    };

    fetchThemeSetting();
}, []);
```

# `update_settings_field` endpoint

---
## Parameters
- `key`: A string representing the setting key to update.
- `value`: The new value to assign to the setting (can be any valid JSON value).

## Returns
- Ok(String): A JSON string representation of the updated settings if successful.
- Err(String): An error message if the update operation failed.

## Example call
```typescript jsx
const updateTheme = async () => {
    try {
        const updatedSettings = await invoke("update_settings_field", { 
            key: "theme", 
            value: "dark" 
        });
        console.log("Updated settings:", JSON.parse(updatedSettings));
    } catch (error) {
        console.error("Error updating theme:", error);
    }
};
```

# `update_multiple_settings_command` endpoint

---
## Parameters
- `updates`: A map/object of setting keys to their new values.

## Returns
- Ok(String): A JSON string representation of the updated settings if successful.
- Err(String): An error message if the update operation failed.

## Example call
```typescript jsx
const updateMultipleSettings = async () => {
    try {
        const updates = {
            "theme": "dark",
            "notifications": true,
            "language": "en"
        };
        
        const updatedSettings = await invoke("update_multiple_settings_command", { 
            updates: updates 
        });
        console.log("Updated settings:", JSON.parse(updatedSettings));
    } catch (error) {
        console.error("Error updating settings:", error);
    }
};
```

# `reset_settings` endpoint

---
## Parameters
- `None`

## Returns
- Ok(()): If the settings file was successfully reset.
- Err(String): An error message if the reset failed.

## Example call
```typescript jsx
const resetSettings = async () => {
    try {
        await invoke("reset_settings");
        console.log("Settings reset to default.");
    } catch (error) {
        console.error("Failed to reset settings:", error);
    }
};
```