import {useEffect, useState} from "react";
import {invoke} from "@tauri-apps/api/tauri";
import {ContextMenuType, DirectoryContent, Volume} from "./types";
import {openDirectory} from "./ipc/fileExplorer";
import VolumeList from "./components/MainBody/Volumes/VolumeList";
import FolderNavigation from "./components/TopBar/FolderNavigation";
import {DirectoryContents} from "./components/MainBody/DirectoryContents";
import useNavigation from "./hooks/useNavigation";
import SearchBar from "./components/TopBar/SearchBar";
import ContextMenu from "./components/ContextMenu";
import {useAppDispatch, useAppSelector} from "./state/hooks";
import {selectCurrentContextMenu} from "./state/slices/contextMenuSlice";
import useContextMenu from "./hooks/useContextMenu";

function App() {
  const [volumes, setVolumes] = useState<Volume[]>([]);
  const [directoryContents, setDirectoryContents] = useState<
      DirectoryContent[]
  >([]);

  const [searchResults, setSearchResults] = useState<DirectoryContent[]>([]);

  const {
    pathHistory,
    historyPlace,
    setHistoryPlace,
    onBackArrowClick,
    onForwardArrowClick,
    canGoBackward,
    canGoForward,
    currentVolume,
    setCurrentVolume,
  } = useNavigation(searchResults, setSearchResults);

  async function updateDirectoryContents() {
    const contents = await openDirectory(pathHistory[historyPlace]);
    setDirectoryContents(contents);
  }

  async function onVolumeClick(mountpoint: string) {
    if (pathHistory[pathHistory.length - 1] != mountpoint) {
      pathHistory.push(mountpoint);
    }
    setHistoryPlace(pathHistory.length - 1);
    setCurrentVolume(mountpoint);

    const directoryContents = await openDirectory(pathHistory[historyPlace]);
    setDirectoryContents(directoryContents);
  }

  async function onDirectoryClick(filePath: string) {
    if (searchResults.length > 0) {
      setSearchResults([]);
    }

    pathHistory.push(filePath);
    setHistoryPlace(pathHistory.length - 1);

    await updateDirectoryContents();
  }

  async function getVolumes() {
    if (volumes.length > 0) {
      return;
    }

    const newVolumes = await invoke<Volume[]>("get_volumes");
    setVolumes(newVolumes);
  }

  let render = 0;

  useEffect(() => {
    if (render === 0) {
      getVolumes().catch(console.error);
    }

    render += 1; // I don't know why but the use effect runs twice causing the "get_volumes" to be called twice.
  }, [])

  useEffect(() => {
    if (pathHistory[historyPlace] == "") {
      setCurrentVolume("");
      return;
    }

    updateDirectoryContents().catch(console.error);
  }, [historyPlace]);

  const dispatch = useAppDispatch();
  const [handleContextMenu, handleCloseContextMenu] = useContextMenu(dispatch);

  const currentContextMenu = useAppSelector(selectCurrentContextMenu);

  return (
    <div className="h-full" onClick={handleCloseContextMenu} onContextMenu={handleContextMenu}>
      {currentContextMenu === ContextMenuType.General ? (
          <ContextMenu options={[
            { name: "General Opt 1", onClick: () => {} },
            { name: "General Opt 2", onClick: () => {} }
          ]} />
      ) : currentContextMenu === ContextMenuType.DirectoryEntity ? (
          <ContextMenu options={[
            { name: "Entity Opt 1", onClick: () => {} },
            { name: "Entity Opt 2", onClick: () => {} }
          ]} />
      ) : ""}

      <div className="p-4">
        <div className="flex justify-between pb-5">
          <FolderNavigation
              onBackArrowClick={onBackArrowClick}
              canGoBackward={canGoBackward()}
              onForwardArrowClick={onForwardArrowClick}
              canGoForward={canGoForward()}
          />

          <SearchBar
              currentVolume={currentVolume}
              currentDirectoryPath={pathHistory[historyPlace]}
              setSearchResults={setSearchResults}
          />
        </div>

        {pathHistory[historyPlace] === "" && searchResults.length === 0 ? (
            <VolumeList volumes={volumes} onClick={onVolumeClick} />
        ) : (
            <DirectoryContents
                content={
                  searchResults.length === 0 ? directoryContents : searchResults
                }
                onDirectoryClick={onDirectoryClick}
            />
        )}
      </div>
    </div>
  );
}

export default App;
