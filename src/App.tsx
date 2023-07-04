import {useEffect, useState} from "react";
import {invoke} from "@tauri-apps/api/tauri";
import {DirectoryContent, Volume} from "./types";
import {openDirectory} from "./ipc";
import VolumeList from "./components/MainBody/Volumes/VolumeList";
import FolderNavigation from "./components/TopBar/FolderNavigation";
import {DirectoryContents} from "./components/MainBody/DirectoryContents";
import useNavigation from "./hooks/useNavigation";
import SearchBar from "./components/TopBar/SearchBar";
import {useAppDispatch, useAppSelector} from "./state/hooks";
import useContextMenu from "./hooks/useContextMenu";
import ContextMenus from "./components/ContextMenus/ContextMenus";
import {
  selectDirectoryContents,
  unselectDirectoryContents,
  updateDirectoryContents
} from "./state/slices/currentDirectorySlice";
import {DIRECTORY_ENTITY_ID} from "./components/MainBody/DirectoryEntity";

function App() {
  const [volumes, setVolumes] = useState<Volume[]>([]);
  const directoryContents = useAppSelector(selectDirectoryContents);
  const dispatch = useAppDispatch();

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

  async function getNewDirectoryContents() {
    const contents = await openDirectory(pathHistory[historyPlace]);
    dispatch(updateDirectoryContents(contents));
  }

  async function onVolumeClick(mountpoint: string) {
    if (pathHistory[pathHistory.length - 1] != mountpoint) {
      pathHistory.push(mountpoint);
    }
    setHistoryPlace(pathHistory.length - 1);
    setCurrentVolume(mountpoint);

    await getNewDirectoryContents();
  }

  async function onDirectoryClick(filePath: string) {
    if (searchResults.length > 0) {
      setSearchResults([]);
    }

    pathHistory.push(filePath);
    setHistoryPlace(pathHistory.length - 1);

    await getNewDirectoryContents();
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

    getNewDirectoryContents().catch(console.error);
  }, [historyPlace]);

  const [handleMainContextMenu, handleCloseContextMenu] = useContextMenu(dispatch, pathHistory[historyPlace]);

  return (
    <div className="h-full" onClick={(e) => {
      handleCloseContextMenu(e);

      if (e.target instanceof HTMLElement) {
        if (e.target.id === DIRECTORY_ENTITY_ID) return;
      }

      dispatch(unselectDirectoryContents());
    }} onContextMenu={handleMainContextMenu}>
      <ContextMenus />

      <div className="p-4">
        <FolderNavigation
            onBackArrowClick={onBackArrowClick}
            canGoBackward={canGoBackward()}
            onForwardArrowClick={onForwardArrowClick}
            canGoForward={canGoForward()}
        />

        <div className="pb-5">
          <SearchBar
              currentVolume={currentVolume}
              currentDirectoryPath={pathHistory[historyPlace]}
              setSearchResults={setSearchResults}
          />

          <div className="w-7/12">
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

      </div>
    </div>
  );
}

export default App;
