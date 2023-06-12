interface Props {
    name: string;
}

export default function File({ name }: Props) {
    return (
        <>{name}<br/></>
    )
}