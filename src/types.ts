export interface Volume {
    name: string;
    mountpoint: string;
    available_gb: number;
    used_gb: number;
    total_gb: number;
}

export type DirectoryEntityType = "file" | "directory";
export type DirectoryContentType = "File" | "Directory";

export interface DirectoryContent {
    [key: string]: [string, string]; // Key will be either "Directory" or "File"
}

export enum ContextMenuType {
    None,
    General,
    DirectoryEntity,
}
