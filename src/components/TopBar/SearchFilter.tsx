import Input, {InputSize} from "../../ui/Input";
import {ChangeEvent, Dispatch, SetStateAction, useState} from "react";
import {DirectoryContent} from "../../types";

interface Props {
    extValue: string;
    setExtValue: Dispatch<SetStateAction<string>>;
    acceptFilesValue: boolean;
    setAcceptFilesValue: Dispatch<SetStateAction<boolean>>;
    acceptDirsValue: boolean;
    setAcceptDirsValue: Dispatch<SetStateAction<boolean>>;
}

export default function SearchFilter({ extValue, setExtValue, acceptFilesValue, setAcceptFilesValue, acceptDirsValue, setAcceptDirsValue }: Props) {
    function onAcceptFilesChange(e: ChangeEvent<HTMLInputElement>) {
        setAcceptFilesValue(e.target.checked);
        if (!e.target.checked && !acceptDirsValue) {
            setAcceptDirsValue(true);
        }
    }

    function onAcceptDirsChange(e: ChangeEvent<HTMLInputElement>) {
        setAcceptDirsValue(e.target.checked);
        if (!e.target.checked && !acceptFilesValue) {
            setAcceptFilesValue(true);
        }
    }

    return (
        <div className="space-x-2 flex justify-center bg-darker p-4 rounded-bl-lg rounded-br-lg w-62">
            <div className="flex flex-col space-y-2">
                <label>Extension</label>
                <label>Files</label>
                <label>Folders</label>
            </div>

            <div className="flex flex-col space-y-2 relative">
                <Input value={extValue} setValue={setExtValue} placeholder="ext" size={InputSize.Tiny} disabled={!acceptFilesValue} />
                <input
                    checked={acceptFilesValue}
                    onChange={onAcceptFilesChange}
                    className="absolute left-2 top-8" type="checkbox"
                />
                <input
                    checked={acceptDirsValue}
                    onChange={onAcceptDirsChange}
                    className="absolute left-2 top-16" type="checkbox"
                />
            </div>
        </div>
    )
}