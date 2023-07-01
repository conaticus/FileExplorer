import { createSlice } from "@reduxjs/toolkit";
import type { PayloadAction } from "@reduxjs/toolkit";
import {ContextMenuType} from "../../types";
import {RootState} from "../store";

interface ContextMenuState {
    currentContextMenu: ContextMenuType,
    mouseX: number;
    mouseY: number;
}

const initialState: ContextMenuState = { currentContextMenu: ContextMenuType.None, mouseX: 0, mouseY: 0, };

export const contextMenuSlice = createSlice({
    name: "contextMenu",
    initialState,
    reducers: {
        updateContextMenu: (state, action: PayloadAction<ContextMenuState>) => {
            state.currentContextMenu = action.payload.currentContextMenu;
            state.mouseX = action.payload.mouseX;
            state.mouseY = action.payload.mouseY;
        },
    }
})

export const { updateContextMenu } = contextMenuSlice.actions;
export const selectCurrentContextMenu = (state: RootState) => state.contextMenu.currentContextMenu;
export default contextMenuSlice.reducer;