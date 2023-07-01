import {useEffect, useState} from "react";

export interface IContextMenu {
    open: boolean;
    mouseX: number;
    mouseY: number;
}

export default function useContextMenu(containerId: string | undefined): IContextMenu {
    const [contextMenu, setContextMenu] = useState<IContextMenu>({
        open: false,
        mouseX: 0,
        mouseY: 0,
    });

    useEffect(() => {
        const container = (containerId ? document.getElementById(containerId) : window) as HTMLElement | undefined;
        container?.addEventListener("contextmenu", (e) => {
            e.preventDefault();

            setContextMenu({
                open: true,
                mouseX: e.pageX,
                mouseY: e.pageY
            });
        })

        addEventListener("click", (e) => {
            if (e.target instanceof Element) {
                if (document.getElementById("context-menu")?.contains(e.target)) { return }
            }

            setContextMenu({
                ...contextMenu,
                open: false,
            })
        })
    }, [])

    return contextMenu;
}