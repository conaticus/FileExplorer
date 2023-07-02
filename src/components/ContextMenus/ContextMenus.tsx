import {ContextMenuType, DirectoryContent} from "../../types";
import ContextMenu from "./ContextMenu";
import {useAppDispatch, useAppSelector} from "../../state/hooks";
import InputModal from "../InputModal";
import {useState} from "react";
import {confirm} from "@tauri-apps/api/dialog";
import {
    ContextMenuState,
    DirectoryEntityContextPayload,
    GeneralContextPayload
} from "../../state/slices/contextMenuSlice";
import {createFile} from "../../ipc";
import {addContent, selectContentIdx} from "../../state/slices/currentDirectorySlice";

export default function ContextMenus() {
    const { currentContextMenu, contextMenuPayload } = useAppSelector(state => state.contextMenu);
    const [newFileShown, setNewFileShown] = useState(false);
    const [newDirectoryShown, setNewDirectoryShown] = useState(false);
    const [renameFileShown, setRenameFileShown] = useState(false);

    // Typescript pain
    const directoryEntityPayload = contextMenuPayload as DirectoryEntityContextPayload;
    const generalPayload = contextMenuPayload as GeneralContextPayload;

    const dispatch = useAppDispatch();

    async function onNewFile(name: string) {
        try {
            const path = generalPayload.currentPath + name;
            await createFile(path);

            const newDirectoryContent = {"File": [name, path]} as DirectoryContent;
            dispatch(addContent(newDirectoryContent));
            dispatch(selectContentIdx(0)) // Select top item as content is added to the top.
        } catch (e) {
            alert(e);
        }
    }

    function onNewFolder(name: string) {

    }

    function onRename(name: string) {

    }

    return (
        <>
            {currentContextMenu == ContextMenuType.General ? (
                <ContextMenu options={[
                    { name: "New File", onClick: () => setNewFileShown(true) },
                    { name: "New Folder", onClick: () => setNewDirectoryShown(true)}
                ]} />
            ) : currentContextMenu == ContextMenuType.DirectoryEntity ? (
                <ContextMenu options={[
                    { name: "Rename", onClick: () => setRenameFileShown(true) },
                    { name: "Delete", onClick: async () => {
                        const result = await confirm("Are you sure you want to delete this file?");
                        if (result) {
                            // Delete file
                        }
                    }}
                ]} />
            ) : ""}

            <InputModal shown={newFileShown} setShown={setNewFileShown} title="New File" onSubmit={onNewFile} submitName="Create" />
            <InputModal shown={newDirectoryShown} setShown={setNewDirectoryShown} title="New Folder" onSubmit={onNewFolder} submitName="Create" />
            <InputModal shown={renameFileShown} setShown={setRenameFileShown} title="Rename File" onSubmit={onRename} submitName="Rename" />
        </>
    )
}
