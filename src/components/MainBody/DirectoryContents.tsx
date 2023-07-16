import DirectoryEntity from "./DirectoryEntity";
import {DirectoryContent, DirectoryContentType} from "../../types";
import {openFile, paste} from "../../ipc";
import {useEffect, useRef} from "react";
import { copyFile, stat } from "fs";
import { clipboard } from "@tauri-apps/api";
import { useAppDispatch } from "../../state/hooks";
import { addContent, selectContentIdx } from "../../state/slices/currentDirectorySlice";
import { createDirectoryContent } from "../../util";
import {confirm} from "@tauri-apps/api/dialog";

interface Props {
    content: DirectoryContent[];
    onDirectoryClick: (filePath: string) => any;
    path: string;
}

export function DirectoryContents({content, onDirectoryClick, path}: Props) {
    const dispatch = useAppDispatch();

    async function onFileClick(path: string) {
        await openFile(path).catch(err => alert(err));
    }

    function useKey(key: string, cb: Function){
        const callback = useRef(cb);

        useEffect(() => {
            callback.current = cb;
        })
    
    
        useEffect(() => {
            function handle(event: KeyboardEvent){
                if(event.code === key){
                    callback.current(event);
                } else if (key === 'ctrlc' && event.key === 'c' && event.ctrlKey) {
                    callback.current(event);
                } else if (key === 'ctrlv' && event.key === 'v' && event.ctrlKey) {
                    callback.current(event);
                }
            }
    
            document.addEventListener('keydown',handle);
            return () => document.removeEventListener("keydown",handle)
        },[key])
    }

    useKey('ctrlc', (e: KeyboardEvent) => {
        e.preventDefault();
        const selected = document.getElementById("selected");
        if (selected) {
            if (!path.endsWith("/")) {
                path += "/";
            }
            clipboard.writeText(path + selected.innerText.replace(/(\r\n|\n|\r)/gm, ""));
        }
    })

    useKey('ctrlv', async (e: KeyboardEvent) => {
        e.preventDefault();
        var overwrite = false;
        var exists = false;
        var source = await clipboard.readText() as string;
        const name = source.split("/").pop();

        // check if path already exists in current directory
        const parent = document.getElementById("entries");
        if (parent) {
            const children = parent.children;
            for (let i = 0; i < children.length; i++) {
                const child = children[i];
                if (child.hasAttribute("title") && child.getAttribute("title") === name) {
                    exists = true;
                    // ask user if they want to overwrite
                    const result = await confirm("A file or directory with this name already exists in this directory. Would you like to overwrite it?")

                    if (result == true) {
                        overwrite = true;
                    }
                }
            }
        }
        if (!overwrite && exists) {
            return;
        }
        const destination = path
        const type = await paste(source, destination);

        if (exists == false) {
            const newDirectoryContent = createDirectoryContent(type, name, destination + "/" + name);
            dispatch(addContent(newDirectoryContent));
            dispatch(selectContentIdx(0)) // Select top item as content is added to the top.
        }

    })


    return <>
        <div id="entries">
            {content.length === 0 ? "There are no files in this directory." : ""}

            {content.map((content, idx) => {
                const [fileType, [fileName, filePath]] = Object.entries(content)[0];

                return (
                    <DirectoryEntity
                        type={fileType as DirectoryContentType}
                        onDoubleClick={() =>
                            fileType === "Directory"
                                ? onDirectoryClick(filePath)
                                : onFileClick(filePath)
                        }

                        key={idx}
                        idx={idx}
                        name={fileName}
                        path={filePath}
                    />
                );
            })}
        </div>
    </>;
}