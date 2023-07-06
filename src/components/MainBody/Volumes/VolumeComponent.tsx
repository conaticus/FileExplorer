import {Volume} from "../../../types";
import {MouseEventHandler} from "react";

interface Props {
    volume: Volume;
    onClick: MouseEventHandler<HTMLButtonElement>;
}

export default function VolumeComponent({ volume, onClick }: Props) {
    return (
        <button onClick={onClick} className="p-4 bg-zinc-900 rounded-xl cursor-pointer justify-between flex flex-col">
            <h3>{volume.name} ({volume.mountpoint})</h3>

            <div className="mt-2">
            <progress max="100" className={`${((volume.used_gb / volume.total_gb) * 100) > 50 && ((volume.used_gb / volume.total_gb) * 100) < 90 ? "mid" : ""} ${((volume.used_gb / volume.total_gb) * 100) > 90 ? "full" : ""}`} value={(volume.used_gb / volume.total_gb) * 100} />
            <p>{volume.available_gb} GB free of {volume.total_gb} GB</p>
            </div>
        </button>
    )
}
