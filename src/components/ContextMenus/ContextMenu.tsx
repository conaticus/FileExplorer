import {useAppDispatch, useAppSelector} from "../../state/hooks";
import {updateContextMenu} from "../../state/slices/contextMenuSlice";
import {NO_CONTEXT_MENU} from "../../state/constants/constants";

interface ContextMenuOption {
    name: string;
    onClick: Function;
}

interface Props {
   options: ContextMenuOption[];
}

export default function ContextMenu({ options }: Props) {
    const dispatch = useAppDispatch();
    const { mouseX, mouseY } = useAppSelector(state => state.contextMenu);

    return (
        <div id="context-menu" className="bg-darker w-48" style={{
            position: "absolute",
            left: mouseX,
            top: mouseY,
        }}>
            {options.map((option, idx) => (
                <div key={idx} className="">
                    <button onClick={() => {
                        option.onClick();
                        dispatch(updateContextMenu(NO_CONTEXT_MENU));
                    }} className="bg-darker hover:bg-bright w-full">{option.name}</button>
                    <br />
                </div>
            ))}
        </div>
    )
}