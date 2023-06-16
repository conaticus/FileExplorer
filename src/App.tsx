import {useEffect, useState} from "react";
import { invoke } from "@tauri-apps/api/tauri";
import {DirectoryContent, Disk} from "./types";
import DiskComponent from "./components/DiskComponent";
import {openDirectory} from "./ipc/fileExplorer";
import DirectoryEntity from "./components/DirectoryEntity";
import {FontAwesomeIcon} from "@fortawesome/react-fontawesome";
import { faArrowLeft, faArrowRight } from "@fortawesome/free-solid-svg-icons";

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

    function canGoForward(): boolean {
        return historyPlace < pathHistory.length - 1;
    }

    function canGoBackward(): boolean {
        return historyPlace > 0;
    }

    function onBackArrowClick() {
        pathHistory.push(pathHistory[historyPlace - 1]);
        setHistoryPlace(historyPlace - 1);
    }

    function onForwardArrowClick() {
        setHistoryPlace(historyPlace + 1);
    }

    useEffect(() => {
        getData().catch(console.error);
    }, [])

    async function updateCurrentDirectory() {
        console.log(pathHistory)
        if (pathHistory[historyPlace] == "") {
            return getData();
        }

        const directoryContents= await openDirectory(pathHistory[historyPlace]);
        setDirectoryContents(directoryContents);
    }

    useEffect(() => {
        updateCurrentDirectory();
    }, [historyPlace])

    return (
        <div className="p-4">
            <div className="mb-5">
                <div className="space-x-4">
                    <button onClick={onBackArrowClick} disabled={!canGoBackward()}>
                        <FontAwesomeIcon
                            icon={faArrowLeft}
                            size="xl"
                            className={canGoBackward() ? undefined : "text-gray-600"}
                        />
                    </button>

                    <button onClick={onForwardArrowClick} disabled={!canGoForward()}>
                        <FontAwesomeIcon icon={faArrowRight} size="xl" className={canGoForward() ? undefined : "text-gray-600"} />
                    </button>
                </div>
            </div>

            {pathHistory[historyPlace] === "" ? (
                <div className="space-x-4">
                    {disks.map((disk, idx) => (
                        <DiskComponent
                            onClick={() => onDiskClick(disk.letter)}
                            disk={disk}
                            key={idx}
                        />
                    ))}
                </div>
            ) : (
                <>
                    {directoryContents.length === 0 ? "There are no files in this directory." : ""}

                    {directoryContents.map((content, idx) => {
                        const [fileType, fileName] = Object.entries(content)[0];

                        return (
                            <DirectoryEntity
                                type={fileType === "Directory" ? "directory" : "file"}
                                onClick={() =>
                                    fileType === "Directory"
                                        ? onDirectoryClick(fileName)
                                        : undefined
                                }
                                key={idx}
                                name={fileName}
                            />
                        );
                    })}
                </>
            )}
        </div>
    );
}

export default App;
