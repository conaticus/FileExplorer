import VolumeComponent from "./VolumeComponent";
import {Volume} from "../../../types";
import LoadingPlaceholder from "../Util/LoadingPlaceholder";
import { Fragment } from "react";

interface Props {
    volumes: Volume[];
    onClick: (mountpoint: string) => any;
}

export default function VolumeList({ volumes, onClick }: Props) {
    return (
        <Fragment>
        <div className={`${volumes.length == 0 ? "opacity-0 translate-y-96" : "opacity-100 translate-y-0"} grid grid-cols-4 gap-4 transition-all duration-300 ease-out`}>
            {volumes.length != 0 && volumes.map((volume, idx) => (
                <VolumeComponent
                    onClick={() => onClick(volume.mountpoint)}
                    volume={volume}
                    key={idx}
                />
            ))}
        </div>
        <div className={`w-full h-full flex flex-col items-center justify-center ${volumes.length == 0 ? "opacity-100" : "opacity-0"} transition-all duration-300 ease-in-out`}>
            <LoadingPlaceholder/>
        </div>

        </Fragment>
    )
}
