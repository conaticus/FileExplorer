# Tauri Template Commands Documentation

## Content
- [Get Template Paths as JSON](#get_template_paths_as_json-endpoint)
- [Add Template](#add_template-endpoint)
- [Use Template](#use_template-endpoint)
- [Remove Template](#remove_template-endpoint)

# `get_template_paths_as_json` endpoint

---
## Parameters
- None

## Returns
- `Ok(String)` - A JSON array of template paths as strings.
- `Err(String)` - An error message if the templates can't be retrieved.

## Description
Retrieves all available templates as a JSON string of paths. These templates are stored in the application's template directory.

## Example call
```typescript jsx
useEffect(() => {
    const fetchTemplatePaths = async () => {
        try {
            const jsonPaths = await invoke("get_template_paths_as_json");
            const templatePaths = JSON.parse(jsonPaths);
            console.log("Available templates:", templatePaths);
        } catch (error) {
            console.error("Error fetching template paths:", error);
        }
    };

    fetchTemplatePaths();
}, []);
```

# `add_template` endpoint

---
## Parameters
- `template_path`: A string representing the absolute path to the file or directory to be added as a template.

## Returns
- `Ok(String)` - A success message including the name of the template and its size.
- `Err(String)` - An error message if the template cannot be added.

## Description
Adds a template to the application's template directory. This function copies a file or directory from the provided path and registers it as a template. The original file/directory remains unchanged.

## Example call
```typescript jsx
const addTemplate = async () => {
    try {
        const result = await invoke("add_template", { 
            template_path: "/path/to/my/template" 
        });
        console.log("Template added:", result);
    } catch (error) {
        console.error("Error adding template:", error);
    }
};
```

# `use_template` endpoint

---
## Parameters
- `template_path`: A string representing the absolute path to the template.
- `dest_path`: A string representing the absolute path where the template should be applied.

## Returns
- `Ok(String)` - A success message with details about the template application.
- `Err(String)` - An error message if the template cannot be applied.

## Description
Applies a template to the specified destination path. This function copies the content of a template (file or directory) to the specified destination. The template remains unchanged, creating a new instance at the destination path.

## Example call
```typescript jsx
const applyTemplate = async () => {
    try {
        const result = await invoke("use_template", { 
            template_path: "/path/to/templates/my_template",
            dest_path: "/path/where/to/apply/template"
        });
        console.log("Template applied:", result);
    } catch (error) {
        console.error("Error applying template:", error);
    }
};
```

# `remove_template` endpoint

---
## Parameters
- `template_path`: A string representing the absolute path to the template to be removed.

## Returns
- `Ok(String)` - A success message confirming the removal of the template.
- `Err(String)` - An error message if the template cannot be removed.

## Description
Removes a template from the application's template directory. This function deletes a template (file or directory) and updates the registered templates list.

## Example call
```typescript jsx
const removeTemplate = async () => {
    try {
        const result = await invoke("remove_template", { 
            template_path: "/path/to/templates/my_template" 
        });
        console.log("Template removed:", result);
    } catch (error) {
        console.error("Error removing template:", error);
    }
};
```
