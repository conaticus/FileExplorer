import {DirectoryContent, DirectoryContentType} from "./types";

export function removeFileNameFromPath(path: string): string {
    return path.substring(0, path.lastIndexOf("\\"));
}

export function createDirectoryContent(type: DirectoryContentType, name: string, path: string): DirectoryContent {
   return {[type]: [name, path]};
}