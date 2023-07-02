import {useEffect, useState} from "react";
import {invoke} from "@tauri-apps/api/tauri";
import {DirectoryContent, Volume} from "./types";
import {openDirectory} from "./ipc/fileExplorer";
import VolumeList from "./components/MainBody/Volumes/VolumeList";
import FolderNavigation from "./components/TopBar/FolderNavigation";
import {DirectoryContents} from "./components/MainBody/DirectoryContents";
import useNavigation from "./hooks/useNavigation";
import SearchBar from "./components/TopBar/SearchBar";
import {useAppDispatch} from "./state/hooks";
import useContextMenu from "./hooks/useContextMenu";
import ContextMenus from "./components/ContextMenus/ContextMenus";

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
  const [handleMainContextMenu, handleCloseContextMenu] = useContextMenu(dispatch);

  return (
    <div className="h-full" onClick={handleCloseContextMenu} onContextMenu={handleMainContextMenu}>
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

          <div className="w-full">
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
