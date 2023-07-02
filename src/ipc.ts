import {DirectoryContent} from "./types";
import {invoke} from "@tauri-apps/api/tauri";

export async function openDirectory(path: string): Promise<DirectoryContent[]> {
   return invoke("open_directory", { path });
}

export async function openFile(path: string): Promise<string> {
   return invoke<string>("open_file", { path });
}

export async function createFile(path: string): Promise<void> {
   return invoke("create_file", { path });
}

export async function createDirectory(path: string): Promise<void> {
   return invoke("create_directory", { path });
}

export async function renameFile(oldPath: string, newPath: string): Promise<void> {
   return invoke("rename_file", { oldPath, newPath });
}

export async function deleteFile(path: string): Promise<void> {
   return invoke("delete_file", { path });
}

