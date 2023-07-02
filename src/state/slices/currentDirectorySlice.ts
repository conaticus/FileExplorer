import { createSlice } from "@reduxjs/toolkit";
import type { PayloadAction } from "@reduxjs/toolkit";
import {DirectoryContent} from "../../types";
import {RootState} from "../store";
import _ from "lodash";

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
        renameContent: (state, action: PayloadAction<[DirectoryContent, DirectoryContent]>) => {
            const [oldContent, newContent] = action.payload;
            state.contents = state.contents.filter(c => !_.isEqual(c, oldContent));
            state.contents = [newContent, ...state.contents];
        },
        deleteContent: (state, action: PayloadAction<DirectoryContent>) => {
            state.contents = state.contents.filter(c => !_.isEqual(c, action.payload));
        }
    }
})

export const { updateDirectoryContents, unselectDirectoryContents, selectContentIdx, addContent, renameContent, deleteContent } = currentDirectorySlice.actions;
export const selectDirectoryContents = (state: RootState) => state.currentDirectory.contents;
export const selectCurrentSelectedContentIdx = (state: RootState) => state.currentDirectory.currentSelectedIdx;
export default currentDirectorySlice.reducer;