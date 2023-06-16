import {Disk} from "../../types";
import {MouseEventHandler} from "react";

interface Props {
    disk: Disk;
    onClick: MouseEventHandler<HTMLButtonElement>;
}

export default function DiskComponent({ disk, onClick }: Props) {
    return (
        <button onClick={onClick} className="p-5 w-56 bg-darker radius rounded cursor-pointer">
            <h3>{disk.name} ({disk.letter}:)</h3>
            <progress max="100" value={(disk.used_gb / disk.total_gb) * 100} /> <br/>
            {disk.available_gb} GB free of {disk.total_gb} GB
        </button>
    )
}
