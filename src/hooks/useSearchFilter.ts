import {useState} from "react";

export default function useSearchFilter() {
    const [extValue, setExtValue] = useState<string>("");
    const [acceptFilesValue, setAcceptFilesValue] = useState<boolean>(true);
    const [acceptDirsValue, setAcceptDirsValue] = useState<boolean>(true);

    return {
        extValue,
        setExtValue,
        acceptFilesValue,
        setAcceptFilesValue,
        acceptDirsValue,
        setAcceptDirsValue
    }
}