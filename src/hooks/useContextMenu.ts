import {MouseEvent} from "react";
import {updateContextMenu} from "../state/slices/contextMenuSlice";
import {ContextMenuType} from "../types";
import {NO_CONTEXT_MENU} from "../state/constants/constants";
import {AppDispatch} from "../state/store";
import {DIRECTORY_ENTITY_ID} from "../components/MainBody/DirectoryEntity";

export default function useContextMenu(dispatch: AppDispatch, currentPath: string) {
    function handleMainContextMenu(e: MouseEvent<HTMLDivElement>)  {
        e.preventDefault();

        if (e.target instanceof HTMLElement) {
            if (e.target.id === DIRECTORY_ENTITY_ID) return;
        }

        dispatch(updateContextMenu({
            currentContextMenu: ContextMenuType.General,
            mouseX: e.pageX,
            mouseY: e.pageY,
            contextMenuPayload: { currentPath }
        }));
    }

    function handleCloseContextMenu(e: MouseEvent<HTMLDivElement>) {
        if (e.target instanceof HTMLElement) {
            if (document.getElementById("context-menu")?.contains(e.target)) return;
        }

        dispatch(updateContextMenu(NO_CONTEXT_MENU));
    }

    return [handleMainContextMenu, handleCloseContextMenu];
}