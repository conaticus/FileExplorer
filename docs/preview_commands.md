# Tauri Preview Commands Documentation

Error Structure as json can be found [here](./error_structure.md).

## Content

- [Build Preview](#build_preview-endpoint)

---

# `build_preview` endpoint

Generates a preview of a file or directory, handling various file types including images, text files, PDFs, videos, audio files, and folders.

## Parameters

- `path`: String - The file or directory path to generate a preview for

## Returns

- Ok(PreviewPayload) - A preview payload containing the appropriate preview data based on file type
- Err(String) - An error message if the file/directory doesn't exist or cannot be read

## Preview Types

The command returns different preview payloads based on the file type:

### Image Files
For image files (PNG, JPEG, GIF, WebP), returns:
```typescript
{
  kind: "Image",
  name: string,        // Filename
  data_uri: string,    // Base64 encoded image data with MIME type
  bytes: number        // File size in bytes
}
```

### PDF Files
For PDF files, returns:
```typescript
{
  kind: "Pdf",
  name: string,        // Filename
  path: string         // Full file path for external viewer
}
```

### Video Files
For video files (MP4, MOV), returns:
```typescript
{
  kind: "Video",
  name: string,        // Filename
  path: string         // Full file path for external player
}
```

### Audio Files
For audio files (MP3, WAV), returns:
```typescript
{
  kind: "Audio",
  name: string,        // Filename
  path: string         // Full file path for external player
}
```

### Text Files
For text files and other readable content, returns:
```typescript
{
  kind: "Text",
  name: string,        // Filename
  text: string,        // File content (up to 200,000 characters)
  truncated: boolean   // True if content was truncated
}
```

### Folders
For directories, returns:
```typescript
{
  kind: "Folder",
  name: string,           // Directory name
  entries: FolderEntry[], // List of directory contents (up to 200 items)
  truncated: boolean      // True if listing was truncated
}
```

Where `FolderEntry` is:
```typescript
{
  name: string,     // Entry name
  is_dir: boolean   // True if entry is a directory
}
```

### Unknown Files
For unrecognized file types, returns:
```typescript
{
  kind: "Unknown",
  name: string        // Filename
}
```

### Error Cases
For files that cannot be processed, returns:
```typescript
{
  kind: "Error",
  name: string,       // Filename
  message: string     // Error description
}
```

## Supported File Types

### Images
- PNG (.png)
- JPEG (.jpg, .jpeg)
- GIF (.gif)
- WebP (.webp)

### Documents
- PDF (.pdf)

### Video
- MP4 (.mp4)
- QuickTime (.mov)

### Audio
- MP3 (.mp3)
- WAV (.wav)

### Text
- Markdown (.md)
- Rust (.rs)
- TypeScript (.ts, .tsx)
- JavaScript (.js, .jsx)
- JSON (.json)
- Plain text (.txt)
- Log files (.log)
- Configuration files (.toml, .yaml, .yml, .xml, .ini)
- CSV (.csv)

## Example Usage

```typescript jsx
import { invoke } from '@tauri-apps/api/tauri';

const PreviewComponent = ({ filePath }) => {
  const [preview, setPreview] = useState(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState(null);

  const loadPreview = async (path) => {
    setLoading(true);
    setError(null);
    
    try {
      const result = await invoke("build_preview", { path });
      setPreview(result);
    } catch (err) {
      setError(err);
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    if (filePath) {
      loadPreview(filePath);
    }
  }, [filePath]);

  if (loading) return <div>Loading preview...</div>;
  if (error) return <div>Error: {error}</div>;
  if (!preview) return <div>No preview available</div>;

  // Render based on preview type
  switch (preview.kind) {
    case "Image":
      return (
        <div>
          <h3>{preview.name}</h3>
          <img src={preview.data_uri} alt={preview.name} />
          <p>Size: {preview.bytes} bytes</p>
        </div>
      );

    case "Text":
      return (
        <div>
          <h3>{preview.name}</h3>
          <pre>{preview.text}</pre>
          {preview.truncated && <p><em>Content truncated...</em></p>}
        </div>
      );

    case "Folder":
      return (
        <div>
          <h3>{preview.name}/</h3>
          <ul>
            {preview.entries.map((entry, index) => (
              <li key={index}>
                {entry.is_dir ? "📁" : "📄"} {entry.name}
              </li>
            ))}
          </ul>
          {preview.truncated && <p><em>Listing truncated at 200 items...</em></p>}
        </div>
      );

    case "Pdf":
    case "Video":
    case "Audio":
      return (
        <div>
          <h3>{preview.name}</h3>
          <p>File path: {preview.path}</p>
          <p>Open with external application</p>
        </div>
      );

    case "Unknown":
      return (
        <div>
          <h3>{preview.name}</h3>
          <p>Unknown file type</p>
        </div>
      );

    case "Error":
      return (
        <div>
          <h3>{preview.name}</h3>
          <p>Error: {preview.message}</p>
        </div>
      );

    default:
      return <div>Unsupported preview type</div>;
  }
};

// Usage example
const App = () => {
  const [selectedPath, setSelectedPath] = useState("");

  return (
    <div>
      <input 
        type="text" 
        value={selectedPath}
        onChange={(e) => setSelectedPath(e.target.value)}
        placeholder="Enter file or folder path"
      />
      <PreviewComponent filePath={selectedPath} />
    </div>
  );
};
```

## Performance Considerations

- **Image files**: Large images (>6MB) are truncated to the first 256KB for performance
- **Text files**: Content is limited to 200,000 characters to prevent memory issues
- **Folders**: Directory listings are limited to 200 entries to maintain responsiveness
- **File detection**: Uses both content analysis and file extensions for accurate type detection
- **Encoding detection**: Automatically detects text file encoding for proper UTF-8 conversion

## Error Handling

The command handles various error scenarios:
- File or directory not found
- Permission denied
- Corrupted or unreadable files
- Network issues (for remote files)
- Memory limitations for very large files

All errors are returned as descriptive string messages for easy debugging and user feedback.

## Security Notes

- File paths are validated to prevent directory traversal attacks
- Large files are automatically truncated to prevent memory exhaustion
- Binary files are handled safely without attempting text conversion
- No external network requests are made during preview generation
