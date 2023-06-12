import {MouseEventHandler} from "react";

interface Props {
    name: string;
    onClick: MouseEventHandler<HTMLButtonElement>;
}

export default function Directory({ name, onClick }: Props) {
    return (
        <button onClick={onClick}>
            Directory: {name}
        </button>
    )
}