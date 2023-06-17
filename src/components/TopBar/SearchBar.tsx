import {Dispatch, SetStateAction, useState} from "react";
import {DirectoryContent} from "../../types";
import {invoke} from "@tauri-apps/api/tauri";
import SearchFilter from "./SearchFilter";
import Input, {InputSize} from "../../ui/Input";
import useSearchFilter from "../../hooks/useSearchFilter";

interface Props {
    currentDirectoryPath: string;
    setSearchResults: Dispatch<SetStateAction<DirectoryContent[]>>
}

export default function SearchBar({ currentDirectoryPath, setSearchResults }: Props) {
    const [searchValue, setSearchValue] = useState("");
    const {
        extValue,
        setExtValue,
        acceptFilesValue,
        setAcceptFilesValue,
        acceptDirsValue,
        setAcceptDirsValue
    } = useSearchFilter();

    const split = currentDirectoryPath.split("/");
    const currentPlace = split[split.length - 2];

    async function onSearch() {
        const results = await invoke<DirectoryContent[]>("search_directory", { query: searchValue, searchDirectory: currentDirectoryPath });
        setSearchResults(results);
    }

    return (
        <div>
            <Input value={searchValue} setValue={setSearchValue} placeholder={`Search ${currentPlace || "PC."}`} className="rounded-bl-none rounded-br-none" onSubmit={onSearch} size={InputSize.Large} />
            <SearchFilter
                extValue={extValue}
                setExtValue={setExtValue}
                acceptFilesValue={acceptFilesValue}
                setAcceptFilesValue={setAcceptFilesValue}
                acceptDirsValue={acceptDirsValue}
                setAcceptDirsValue={setAcceptDirsValue}
            />
        </div>
    )
}