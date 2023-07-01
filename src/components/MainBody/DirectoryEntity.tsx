import {MouseEventHandler, MutableRefObject, useRef} from "react";
import {DirectoryEntityType} from "../../types";
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome"
import { faFile, faFolder } from "@fortawesome/free-solid-svg-icons"

interface Props {
    name: string;
    type: DirectoryEntityType;
    onDoubleClick: MouseEventHandler<HTMLButtonElement>;
}

export default function DirectoryEntity({ name, type, onDoubleClick }: Props) {
    const buttonRef = useRef<HTMLButtonElement>(null);

    return (
        <>
            <button
                className="bg-background hover:bg-darker cursor-pointer w-full h-8 flex focus:bg-darker"
                onDoubleClick={(e) => {
                    onDoubleClick(e);
                    buttonRef.current?.blur();
                }}
                ref={buttonRef}
            >
                <div className="mr-1">
                    <FontAwesomeIcon icon={type == "file" ? faFile : faFolder} size="lg" color={type == "file" ? "gray" : "#FFD54F"} />
                </div>
                {name}
            </button>
        </>
    )
}