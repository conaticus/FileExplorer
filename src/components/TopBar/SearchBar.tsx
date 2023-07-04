import {Dispatch, SetStateAction, useEffect, useState} from "react";
import { DirectoryContent } from "../../types";
import { invoke } from "@tauri-apps/api/tauri";
import SearchFilter from "./SearchFilter";
import Input, { InputSize } from "../../ui/Input";

interface Props {
  currentVolume: string;
  currentDirectoryPath: string;
  setSearchResults: Dispatch<SetStateAction<DirectoryContent[]>>;
}

export interface ISearchFilter {
  extension: string;
  acceptFiles: boolean;
  acceptDirectories: boolean;
}

export default function SearchBar({
  currentDirectoryPath,
  currentVolume,
  setSearchResults,
}: Props) {
  const [searchValue, setSearchValue] = useState("");
  const [searchFilter, setSearchFilter] = useState<ISearchFilter>({
    extension: "",
    acceptFiles: true,
    acceptDirectories: true,
  });

  const [currentPlace, setCurrentPlace] = useState<string | undefined>();

  useEffect(() => {
    const split = currentDirectoryPath.split("\\");
    setCurrentPlace(split[split.length - 2]);
  }, [currentDirectoryPath])

  async function onSearch() {
    if (currentVolume.length == 0) {
      alert("Please select a volume before searching.");
      return;
    }

    const results = await invoke<DirectoryContent[]>("search_directory", {
      query: searchValue,
      searchDirectory: currentDirectoryPath,
      mountPnt: currentVolume,
      extension: searchFilter.extension,
      acceptFiles: searchFilter.acceptFiles,
      acceptDirectories: searchFilter.acceptDirectories,
    });

    setSearchResults(results);
  }

  return (
    <div className="absolute right-4 top-4">
      <Input
        value={searchValue}
        setValue={setSearchValue}
        placeholder={`Search ${currentPlace || "PC"}`}
        className="rounded-bl-none rounded-br-none"
        onSubmit={onSearch}
        size={InputSize.Large}
      />
      <SearchFilter filters={searchFilter} setFilters={setSearchFilter} />
    </div>
  );
}
