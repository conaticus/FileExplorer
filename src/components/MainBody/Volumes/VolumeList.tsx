import VolumeComponent from "./VolumeComponent";
import {Volume} from "../../../types";
import LoadingPlaceholder from "../Util/LoadingPlaceholder";

interface Props {
    volumes: Volume[];
    onClick: (mountpoint: string) => any;
}

export default function VolumeList({ volumes, onClick }: Props) {
    return (
        <div className="space-x-4">
            {volumes.length == 0 ? <LoadingPlaceholder/> : volumes.map((volume, idx) => (
                <VolumeComponent
                    onClick={() => onClick(volume.mountpoint)}
                    volume={volume}
                    key={idx}
                />
            ))}
        </div>
    )
}
