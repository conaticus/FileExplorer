import {useEffect, useState} from "react";
import {invoke} from "@tauri-apps/api/tauri";
import {DirectoryContent, Disk} from "./types";
import {openDirectory} from "./ipc/fileExplorer";
import DiskList from "./components/Disks/DiskList";
import FolderNavigation from "./components/FolderNavigation";
import {DirectoryContents} from "./components/DirectoryContents";

function App() {
    const [disks, setDisks] = useState<Disk[]>([]);

    const [pathHistory, setPathHistory] = useState<string[]>([""]);
    const [historyPlace, setHistoryPlace] = useState<number>(0);

    const [directoryContents, setDirectoryContents] = useState<DirectoryContent[]>([]);

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

        const directoryContents= await openDirectory(pathHistory[historyPlace]);
        setDirectoryContents(directoryContents);
    }

    async function getData() {
        const disks = await invoke<Disk[]>("get_disks");
        setDisks(disks);
    }

    function canGoForward(): boolean { return historyPlace < pathHistory.length - 1 }
    function canGoBackward(): boolean { return historyPlace > 0 }

    function onBackArrowClick() {
        pathHistory.push(pathHistory[historyPlace - 1]);
        setHistoryPlace(historyPlace - 1);
    }

    function onForwardArrowClick() {
        setHistoryPlace(historyPlace + 1);
    }

    async function updateCurrentDirectory() {
        if (pathHistory[historyPlace] == "") {
            return getData();
        }

        const directoryContents= await openDirectory(pathHistory[historyPlace]);
        setDirectoryContents(directoryContents);
    }


    useEffect(() => {
        getData().catch(console.error);
    }, [])

    useEffect(() => {
        updateCurrentDirectory();
    }, [historyPlace])

    return (
        <div className="p-4">
            <FolderNavigation onBackArrowClick={onBackArrowClick} canGoBackward={canGoBackward()} onForwardArrowClick={onForwardArrowClick}
                              canGoForward={canGoForward()}/>

            {pathHistory[historyPlace] === "" ? (
                <DiskList disks={disks} onClick={onDiskClick}/>
            ) : (
                <DirectoryContents content={directoryContents} onDirectoryClick={onDirectoryClick}/>
            )}
        </div>
    );
}

export default App;
