import Input, { InputSize } from "../../ui/Input";
import { ChangeEvent, Dispatch, SetStateAction } from "react";
import { ISearchFilter } from "./SearchBar";
interface Props {
  filters: ISearchFilter;
  setFilters: Dispatch<SetStateAction<ISearchFilter>>;
}

export default function SearchFilter({ filters, setFilters }: Props) {
  function onAcceptFilesChange(e: ChangeEvent<HTMLInputElement>) {
    if (!e.target.checked && !filters.acceptDirectories) {
      setFilters({
        ...filters,
        acceptFiles: false,
        acceptDirectories: true,
      });

      return;
    }

    setFilters({
      ...filters,
      acceptFiles: e.target.checked,
    });
  }

  function onAcceptDirsChange(e: ChangeEvent<HTMLInputElement>) {
    if (!e.target.checked && !filters.acceptFiles) {
      setFilters({
        ...filters,
        acceptDirectories: false,
        acceptFiles: true,
      });

      return;
    }

    setFilters({
      ...filters,
      acceptDirectories: e.target.checked,
    });
  }

  function onExtensionChange(e: ChangeEvent<HTMLInputElement>) {
    setFilters({
      ...filters,
      extension: e.target.value,
    });
  }

  return (
    <div className="flex flex-row items-center rounded-xl w-full bg-zinc-700 px-2 h-12 font-semibold">
      <div className="inline-flex items-center rounded-xl bg-zinc-900 px-2 py-1">
        <p className="mr-2 text-zinc-500">Extension</p>
        <input
          className="text-sm font-semibold bg-zinc-800 rounded-md px-2 border-none ring-none ring-transparent border-transparent"
          type="text"
          disabled={!filters.acceptFiles}
          onChange={onExtensionChange}
          value={filters.extension}
          placeholder="ext"
        />
      </div>
      <div className="inline-flex items-center rounded-xl bg-zinc-900 px-2 py-1 ml-2">
        <p className="mr-2 text-zinc-500">Files</p>
        <input
          checked={filters.acceptFiles}
          onChange={onAcceptFilesChange}
          className=""
          type="checkbox"
        />
      </div>
      <div className="inline-flex items-center rounded-xl bg-zinc-900 px-2 py-1 ml-2">
        <p className="mr-2 text-zinc-500">Folders</p>
        <input
          checked={filters.acceptDirectories}
          onChange={onAcceptDirsChange}
          className=""
          type="checkbox"
        />
      </div>
    </div>
  );
}
