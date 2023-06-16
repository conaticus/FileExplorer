export interface Disk {
    name: string;
    letter: string;
    available_gb: number;
    used_gb: number;
    total_gb: number;
}

export type DirectoryEntityType = "file" | "directory";

export interface DirectoryContent {
    [key: string]: string; // Key will be either "Directory" or "File"
}