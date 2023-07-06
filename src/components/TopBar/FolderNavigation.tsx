import { FontAwesomeIcon } from "@fortawesome/react-fontawesome";
import { faArrowLeft, faArrowRight } from "@fortawesome/free-solid-svg-icons";
import { AdjustmentsVerticalIcon } from "@heroicons/react/20/solid";
import Input, { InputSize } from "../../ui/Input";
import { Dispatch, Fragment, SetStateAction, useEffect, useState } from "react";
import { DirectoryContent } from "../../types";
import { invoke } from "lodash";
import SearchFilter from "./SearchFilter";

export interface Props {
  onBackArrowClick: () => void;
  canGoBackward: boolean;
  onForwardArrowClick: () => void;
  canGoForward: boolean;
  currentVolume: string;
  currentDirectoryPath: string;
  setSearchResults: Dispatch<SetStateAction<DirectoryContent[]>>;
}

export interface ISearchFilter {
  extension: string;
  acceptFiles: boolean;
  acceptDirectories: boolean;
}

export default function FolderNavigation({
  onBackArrowClick,
  canGoBackward,
  onForwardArrowClick,
  canGoForward,
  currentVolume,
  currentDirectoryPath,
  setSearchResults,
}: Props) {
  const [searchValue, setSearchValue] = useState("");
  const [searchFilter, setSearchFilter] = useState<ISearchFilter>({
    extension: "",
    acceptFiles: true,
    acceptDirectories: true,
  });
  const [currentPlace, setCurrentPlace] = useState<string | undefined>();
  const [isFilterOpen, setIsFilterOpen] = useState(false);

  useEffect(() => {
    const split = currentDirectoryPath.split("\\");
    setCurrentPlace(split[split.length - 2]);
  }, [currentDirectoryPath]);

  function onSearch() {
    throw new Error("Function not implemented.");
  }

  return (
    <Fragment>
      <div className="mb-5 w-full flex flex-row justify-start items-center">
        <div className="inline-flex mr-2">
          <button onClick={onBackArrowClick} disabled={!canGoBackward}>
            <FontAwesomeIcon
              icon={faArrowLeft}
              size="xl"
              className={canGoBackward ? undefined : "text-zinc-600"}
            />
          </button>
          <button
            onClick={onForwardArrowClick}
            disabled={!canGoForward}
            className="ml-2"
          >
            <FontAwesomeIcon
              icon={faArrowRight}
              size="xl"
              className={canGoForward ? undefined : "text-zinc-600"}
            />
          </button>
        </div>
        <input
          type="text"
          value={currentDirectoryPath || "SUCC"}
          className="px-4 rounded-xl h-8 text-sm bg-zinc-600 text-zinc-400 w-full"
        ></input>
        <div className="inline-flex ml-2">
          <Input
            value={searchValue}
            setValue={setSearchValue}
            placeholder={`Search ${currentPlace || "PC"}`}
            onSubmit={onSearch}
            size={InputSize.Large}
          />
          <button onClick={() => {setIsFilterOpen(!isFilterOpen)}}>
            <AdjustmentsVerticalIcon className="w-6 h-6 text-zinc-400 ml-2" />
          </button>
        </div>
      </div>
      <div className={`${isFilterOpen ? "h-auto opacity-100 mb-2" : "h-0 opacity-0 mb-0"} ml-[3.7rem] transition-all duration-300 ease-in-out`}>
        <SearchFilter filters={searchFilter} setFilters={setSearchFilter} />
      </div>
    </Fragment>
  );
}
