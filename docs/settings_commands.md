# Tauri Settings Commands Documentation

## Content
- [Get All Settings](#get_settings_as_json-endpoint)
- [Get a Specific Setting](#get_setting_field-endpoint)
- [Update a Setting Field](#update_settings_field-endpoint)
- [Update Multiple Settings](#update_multiple_settings_command-endpoint)

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
