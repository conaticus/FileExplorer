import {useEffect, useState} from "react";
import {invoke} from "@tauri-apps/api/tauri";
import {DirectoryContent, Disk} from "./types";
import {openDirectory} from "./ipc/fileExplorer";
import DiskList from "./components/Disks/DiskList";
import FolderNavigation from "./components/FolderNavigation";
import {DirectoryContents} from "./components/DirectoryContents";
import useNavigation from "./hooks/useNavigation";

function App() {
    const [disks, setDisks] = useState<Disk[]>([]);
    const [directoryContents, setDirectoryContents] = useState<DirectoryContent[]>([]);

    const {
        pathHistory,
        setPathHistory,
        historyPlace,
        setHistoryPlace,
        onBackArrowClick,
        onForwardArrowClick,
        canGoBackward,
        canGoForward,
    } = useNavigation();

    async function updateDirectoryContents() {
        const contents = await openDirectory(pathHistory[historyPlace]);
        setDirectoryContents(directoryContents);
    }

    async function onDiskClick(letter: string) {
        const path = letter + ":/";
        if (pathHistory[pathHistory.length - 1] != path) {
            pathHistory.push(path);
        }
        setHistoryPlace(pathHistory.length - 1);

        const directoryContents = await openDirectory(pathHistory[historyPlace]);
        setDirectoryContents(directoryContents);
    }

    async function onDirectoryClick(name: string) {
        const currentPath = pathHistory[pathHistory.length - 1];
        const newPath = currentPath + name + "/";

        pathHistory.push(newPath);
        setHistoryPlace(pathHistory.length - 1);

        updateDirectoryContents();
    }

    async function getDisks() {
        const disks = await invoke<Disk[]>("get_disks");
        setDisks(disks);
    }

    async function updateCurrentDirectory() {
        if (pathHistory[historyPlace] == "") {
            return getDisks();
        }

        await updateDirectoryContents();
    }

    useEffect(() => {
        if (pathHistory[historyPlace] == "") {
            getDisks().catch(console.error);
            return;
        }

        updateCurrentDirectory();
    }, [historyPlace])

    return (
        <div className="p-4">
            <FolderNavigation onBackArrowClick={onBackArrowClick} canGoBackward={canGoBackward()} onForwardArrowClick={onForwardArrowClick}
                              canGoForward={canGoForward()}/>

            {pathHistory[historyPlace] === ""
                ? <DiskList disks={disks} onClick={onDiskClick}/>
                : <DirectoryContents content={directoryContents} onDirectoryClick={onDirectoryClick}/>}
        </div>
    );
}

export default App;
