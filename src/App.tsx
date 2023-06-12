import {useEffect, useState} from "react";
import { invoke } from "@tauri-apps/api/tauri";
import {DirectoryContent, Disk} from "./types";
import DiskComponent from "./components/DiskComponent";
import {openDirectory} from "./ipc/fileExplorer";
import Directory from "./components/Directory";
import File from "./components/File";

function App() {
    const [disks, setDisks] = useState<Disk[]>([]);
    const [currentPath, setCurrentPath] = useState("");
    const [directoryContents, setDirectoryContents] = useState<DirectoryContent[]>([]);

    async function onDiskClick(letter: string) {
        setCurrentPath(letter + ":/");
        const directoryContents = await openDirectory(currentPath);
        setDirectoryContents(directoryContents);
    }

    async function onDirectoryClick(name:string) {
        setCurrentPath(name + "/")
        const directoryContents= await openDirectory(currentPath);
        setDirectoryContents(directoryContents);
    }

    async function getData() {
        const disks = await invoke<Disk[]>("get_disks");
        setDisks(disks);
    }

    useEffect(() => {
        getData().catch(console.error);
    }, [])

    console.log(directoryContents);

    if (currentPath.length == 0) {
       return (
           <div className="flex space-x-2 p-4">
               {disks.map((disk, idx)=> (
                   <DiskComponent onClick={() => onDiskClick(disk.letter)} disk={disk} key={idx} />
               ))}
           </div>
       )
    } else {
        return (
            <>
                {directoryContents.map((content, idx) => {
                    const [fileType, fileName] = Object.entries(content)[0];

                    if (fileType === "Directory") {
                        return <Directory onClick={() => onDirectoryClick(fileName)} key={idx} name={fileName} />;
                    } else {
                        return <File key={idx} name={fileName} />;
                    }
                })}
            </>
        );
    }
}

export default App;
