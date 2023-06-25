import {useEffect, useState} from "react";
import {invoke} from "@tauri-apps/api/tauri";
import {DirectoryContent, Disk} from "./types";
import {openDirectory} from "./ipc/fileExplorer";
import DiskList from "./components/MainBody/Disks/DiskList";
import FolderNavigation from "./components/TopBar/FolderNavigation";
import {DirectoryContents} from "./components/MainBody/DirectoryContents";
import useNavigation from "./hooks/useNavigation";
import SearchBar from "./components/TopBar/SearchBar";

function App() {
    const [disks, setDisks] = useState<Disk[]>([]);
    const [directoryContents, setDirectoryContents] = useState<DirectoryContent[]>([]);

    const [searchResults, setSearchResults] = useState<DirectoryContent[]>([])

    const {
        pathHistory,
        setPathHistory,
        historyPlace,
        setHistoryPlace,
        onBackArrowClick,
        onForwardArrowClick,
        canGoBackward,
        canGoForward,
    } = useNavigation(searchResults, setSearchResults);

    async function updateDirectoryContents() {
        const contents = await openDirectory(pathHistory[historyPlace]);
        setDirectoryContents(contents);
    }

    async function onDiskClick(letter: string) {
        const path = letter + ":\\";
        if (pathHistory[pathHistory.length - 1] != path) {
            pathHistory.push(path);
        }
        setHistoryPlace(pathHistory.length - 1);

        const directoryContents = await openDirectory(pathHistory[historyPlace]);
        setDirectoryContents(directoryContents);
    }

    async function onDirectoryClick(name: string) {
        const currentPath = pathHistory[pathHistory.length - 1];
        const newPath = currentPath + name + "\\"; // Important that we use backslashes as this is the default in Rust (for comparisons)

        pathHistory.push(newPath);
        setHistoryPlace(pathHistory.length - 1);

        updateDirectoryContents();
    }

    async function getDisks() {
        if (disks.length != 0) { return }

        const newDisks = await invoke<Disk[]>("get_disks");
        setDisks(newDisks);
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
            <div className="flex justify-between pb-5">
                <FolderNavigation onBackArrowClick={onBackArrowClick} canGoBackward={canGoBackward()} onForwardArrowClick={onForwardArrowClick}
                                  canGoForward={canGoForward()}/>

                <SearchBar currentDirectoryPath={pathHistory[historyPlace]} setSearchResults={setSearchResults} />
            </div>

            {pathHistory[historyPlace] === "" && searchResults.length === 0
                ? <DiskList disks={disks} onClick={onDiskClick}/>
                : <DirectoryContents content={searchResults.length === 0 ? directoryContents : searchResults} onDirectoryClick={onDirectoryClick}/>}
        </div>
    );
}

export default App;
