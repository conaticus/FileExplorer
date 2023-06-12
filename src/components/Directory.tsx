interface Props {
    name: string;
}

export default function Directory({ name }: Props) {
    return (
        <>Directory: {name}<br /></>
    )
}