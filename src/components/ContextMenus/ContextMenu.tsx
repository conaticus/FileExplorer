import useContextMenu from "../../hooks/useContextMenu";
import {ReactNode} from "react";

interface Props {
    children: ReactNode;
    containerId?: string, // Container in which the context menu will be triggered
}

export default function ContextMenu({ children, containerId }: Props) {
    const contextMenu = useContextMenu(containerId);

    if (contextMenu.open) {
        return ( // ID must be context menu, so it can be ignored if clicked on (see useContextMenu.ts)
            <div id="context-menu" style={{ position: "absolute", left: contextMenu.mouseX, top: contextMenu.mouseY }}>
                {children}
            </div>
        )
    } else {
       return <></>
    }
}