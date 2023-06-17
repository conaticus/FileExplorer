import { useState } from "react";

export default function useNavigation() {
    const [pathHistory, setPathHistory] = useState([""]);
    const [historyPlace, setHistoryPlace] = useState(0);

    function onBackArrowClick() {
        pathHistory.push(pathHistory[historyPlace - 1]);
        setHistoryPlace((prevPlace) => prevPlace - 1);
    }

    function onForwardArrowClick() {
        setHistoryPlace((prevPlace) => prevPlace + 1);
    }

    function canGoForward(): boolean { return historyPlace < pathHistory.length - 1; }
    function canGoBackward(): boolean { return historyPlace > 0; }

    return {
        pathHistory,
        setPathHistory,
        historyPlace,
        setHistoryPlace,
        onBackArrowClick,
        onForwardArrowClick,
        canGoForward,
        canGoBackward,
    }
}