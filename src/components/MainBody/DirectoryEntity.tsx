import {MouseEvent, MouseEventHandler, useRef} from "react";
import {ContextMenuType, DirectoryEntityType} from "../../types";
import {FontAwesomeIcon} from "@fortawesome/react-fontawesome"
import {faFile, faFolder} from "@fortawesome/free-solid-svg-icons"
import {useAppDispatch} from "../../state/hooks";
import {updateContextMenu} from "../../state/slices/contextMenuSlice";

interface Props {
    name: string;
    type: DirectoryEntityType;
    onDoubleClick: MouseEventHandler<HTMLButtonElement>;
}

export default function DirectoryEntity({ name, type, onDoubleClick }: Props) {
    const buttonRef = useRef<HTMLButtonElement | null>(null);
    const dispatch = useAppDispatch();

    function handleContextMenu(e: MouseEvent<HTMLButtonElement>) {
        e.preventDefault();

        dispatch(updateContextMenu({
            currentContextMenu: ContextMenuType.DirectoryEntity,
            mouseX: e.pageX,
            mouseY: e.pageY,
        }))
    }

    return (
        <>
            <button
                id="directory-entity"
                onContextMenu={handleContextMenu}
                className="directory-entity bg-background hover:bg-darker cursor-pointer w-full h-8 flex focus:bg-darker"
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