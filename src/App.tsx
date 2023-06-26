import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/tauri";
import { DirectoryContent, Volume } from "./types";
import { openDirectory } from "./ipc/fileExplorer";
import VolumeList from "./components/MainBody/Volumes/VolumeList";
import FolderNavigation from "./components/TopBar/FolderNavigation";
import { DirectoryContents } from "./components/MainBody/DirectoryContents";
import useNavigation from "./hooks/useNavigation";
import SearchBar from "./components/TopBar/SearchBar";

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
    if (volumes.length != 0) {
      return;
    }

    const newVolumes = await invoke<Volume[]>("get_volumes");
    setVolumes(newVolumes);
  }

  async function updateCurrentDirectory() {
    if (pathHistory[historyPlace] == "") {
      return getVolumes();
    }

    await updateDirectoryContents();
  }

  useEffect(() => {
    if (pathHistory[historyPlace] == "") {
      getVolumes().catch(console.error);
      setCurrentVolume("");
      return;
    }

    updateCurrentDirectory();
  }, [historyPlace]);

  return (
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
  );
}

export default App;
