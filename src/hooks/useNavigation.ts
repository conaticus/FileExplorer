import { useState } from "react";
import { DirectoryContent } from "../types";

export default function useNavigation(
  searchResults: DirectoryContent[],
  setSearchResults: Function
) {
  const [pathHistory, setPathHistory] = useState([""]);
  const [historyPlace, setHistoryPlace] = useState(0);
  const [currentVolume, setCurrentVolume] = useState("");

  function onBackArrowClick() {
    if (searchResults.length > 0) {
      setHistoryPlace(historyPlace);

      setSearchResults([]);
      return;
    }

    pathHistory.push(pathHistory[historyPlace - 1]);
    setHistoryPlace((prevPlace) => prevPlace - 1);
  }

  function onForwardArrowClick() {
    setHistoryPlace((prevPlace) => prevPlace + 1);
  }

  function canGoForward(): boolean {
    return historyPlace < pathHistory.length - 1;
  }
  function canGoBackward(): boolean {
    return historyPlace > 0;
  }

  return {
    pathHistory,
    setPathHistory,
    historyPlace,
    setHistoryPlace,
    onBackArrowClick,
    onForwardArrowClick,
    canGoForward,
    canGoBackward,
    currentVolume,
    setCurrentVolume,
  };
}
