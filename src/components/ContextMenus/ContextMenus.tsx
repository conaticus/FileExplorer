import {ContextMenuType} from "../../types";
import ContextMenu from "./ContextMenu";
import {useAppSelector} from "../../state/hooks";
import {selectCurrentContextMenu} from "../../state/slices/contextMenuSlice";

export default function ContextMenus() {
    const currentContextMenu = useAppSelector(selectCurrentContextMenu);

    switch (currentContextMenu) {
        case ContextMenuType.General: return (
            <ContextMenu options={[
                { name: "General Opt 1", onClick: () => {} },
                { name: "General Opt 2", onClick: () => {} }
            ]} />
        )
        case ContextMenuType.DirectoryEntity: return (
            <ContextMenu options={[
                { name: "Entity Opt 1", onClick: () => {} },
                { name: "Entity Opt 2", onClick: () => {} }
            ]} />
        )
        default: return <></>;
    }
}
