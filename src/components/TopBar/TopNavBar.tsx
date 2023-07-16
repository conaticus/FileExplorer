import {FontAwesomeIcon} from "@fortawesome/react-fontawesome";
import {faArrowLeft, faArrowRight} from "@fortawesome/free-solid-svg-icons";
import { openDirectory, openFile, getType } from "../../ipc";
import { useState } from "react";
import { useAppDispatch } from "../../state/hooks";
import {
    selectContentIdx,
    unselectDirectoryContents,
    updateDirectoryContents
} from "../../state/slices/currentDirectorySlice";

export interface Props {
    onBackArrowClick: () => void;
    canGoBackward: boolean;
    onForwardArrowClick: () => void;
    canGoForward: boolean;
    path: string;
}

export function TopNavBar({ onBackArrowClick, canGoBackward, onForwardArrowClick, canGoForward, path }: Props) {
    const [currentPath, setCurrentPath] = useState(path);
    const dispatch = useAppDispatch();

    async function onSubmit(e: React.FormEvent<HTMLInputElement>) {
        e.preventDefault();
        const newPath = currentPath;
        dispatch(unselectDirectoryContents());
        console.log("New path: " + newPath);
        if (await getType(newPath) == "Directory") {
            const contents = await openDirectory(newPath);
            dispatch(updateDirectoryContents(contents));
        } else {
            const pathOfParent = newPath.substring(0, newPath.lastIndexOf("/"));
            const contents = await openDirectory(pathOfParent);
            dispatch(updateDirectoryContents(contents));
        }
    }


    return <div className="mb-5 w-full navbar">
        <div className="space-x-4 bg-black h-full flex place-items-center" style={{marginLeft: "calc(-50vw + 50%)", marginTop: "0px", paddingTop: "0px"}}>
            <button className="ml-2 float-left" onClick={onBackArrowClick} disabled={!canGoBackward}>
                <FontAwesomeIcon
                    icon={faArrowLeft}
                    size="xl"
                    className={canGoBackward ? undefined : "text-gray-600"}
                />
            </button>

            <button className="float-left" onClick={onForwardArrowClick} disabled={!canGoForward}>
                <FontAwesomeIcon
                    icon={faArrowRight}
                    size="xl"
                    className={canGoForward ? undefined : "text-gray-600"}
                />
            </button>

                {path == "" ?
                    <div></div>
                    :
                    <div className="mx-auto flex w-1/2 h-10 rounded-full bg-gray-900 pt-px place-items-center">
                        <form className="mx-auto w-11/12 rounded-full bg-gray-800 pt-px" onSubmit={onSubmit}>
                            <input type="text" className="mx-auto w-full rounded-full bg-gray-800 pt-px" value={currentPath} id="path-viewer" onChange={
                                (e) => {
                                    setCurrentPath(e.target.value);
                                }
                            }></input>
                        </form>
                    </div>
                }
        </div>
    </div>;
}