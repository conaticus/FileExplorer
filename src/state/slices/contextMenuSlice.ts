import { createSlice } from "@reduxjs/toolkit";
import type { PayloadAction } from "@reduxjs/toolkit";
import {ContextMenuType, DirectoryContentType} from "../../types";

export interface ContextMenuState {
    currentContextMenu: ContextMenuType,
    contextMenuPayload?: any;
    mouseX: number;
    mouseY: number;
}

export interface GeneralContextPayload {
    currentPath: string;
}

export interface DirectoryEntityContextPayload {
    fileName: string;
    filePath: string;
    type: DirectoryContentType;
}

const initialState: ContextMenuState = { currentContextMenu: ContextMenuType.None, mouseX: 0, mouseY: 0, contextMenuPayload: {} };

export const contextMenuSlice = createSlice({
    name: "contextMenu",
    initialState,
    reducers: {
        updateContextMenu: (state, action: PayloadAction<ContextMenuState>) => {
            return {
                ...state, // Ensures we keep the context menu payload when a modal appears and the context menu closes
                ...action.payload
            }
        },
    }
})

export const { updateContextMenu } = contextMenuSlice.actions;
export default contextMenuSlice.reducer;