import { createSlice } from "@reduxjs/toolkit";
import type { PayloadAction } from "@reduxjs/toolkit";
import {ContextMenuType, DirectoryContent} from "../../types";
import {RootState} from "../store";

export interface CurrentDirectoryState {
    contents: DirectoryContent[];
    currentSelectedIdx?: number;
}

const initialState: CurrentDirectoryState = { contents: [] };

export const currentDirectorySlice = createSlice({
    name: "contextMenu",
    initialState,
    reducers: {
        updateDirectoryContents: (state, action: PayloadAction<DirectoryContent[]>) => {
            state.contents = action.payload;
        },
        addContent: (state, action: PayloadAction<DirectoryContent>) => {
            state.contents = [action.payload, ...state.contents];
        },
        selectContentIdx: (state, action: PayloadAction<number>) => {
            state.currentSelectedIdx = action.payload;
        },
        unselectDirectoryContents: (state) => {
            state.currentSelectedIdx = undefined;
        },
    }
})

export const { updateDirectoryContents, unselectDirectoryContents, selectContentIdx, addContent } = currentDirectorySlice.actions;
export const selectDirectoryContents = (state: RootState) => state.currentDirectory.contents;
export const selectCurrentSelectedContentIdx = (state: RootState) => state.currentDirectory.currentSelectedIdx;
export default currentDirectorySlice.reducer;