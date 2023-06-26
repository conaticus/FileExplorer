import {Volume} from "../../../types";
import {MouseEventHandler} from "react";

interface Props {
    volume: Volume;
    onClick: MouseEventHandler<HTMLButtonElement>;
}

export default function VolumeComponent({ volume, onClick }: Props) {
    return (
        <button onClick={onClick} className="p-5 w-56 bg-darker radius rounded cursor-pointer">
            <h3>{volume.name} ({volume.mountpoint})</h3>
            <progress max="100" value={(volume.used_gb / volume.total_gb) * 100} /> <br/>
            {volume.available_gb} GB free of {volume.total_gb} GB
        </button>
    )
}
