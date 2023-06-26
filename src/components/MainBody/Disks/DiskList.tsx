import DiskComponent from "./DiskComponent";
import {Disk} from "../../../types";
import {MouseEventHandler} from "react";

interface Props {
    disks: Disk[];
    onClick: (mountpoint: string) => any;
}

export default function DiskList({ disks, onClick }: Props) {
    return (
        <div className="space-x-4">
            {disks.map((disk, idx) => (
                <DiskComponent
                    onClick={() => onClick(disk.mountpoint)}
                    disk={disk}
                    key={idx}
                />
            ))}
        </div>
    )
}
