import DirectoryEntity from "./DirectoryEntity";
import {DirectoryContent} from "../../types";

interface Props {
    content: DirectoryContent[];
    onDirectoryClick: (fileName: string) => any;
}

export function DirectoryContents({content, onDirectoryClick}: Props) {
    return <>
        {content.length === 0 ? "There are no files in this directory." : ""}

        {content.map((content, idx) => {
            const [fileType, fileName] = Object.entries(content)[0];

            return (
                <DirectoryEntity
                    type={fileType === "Directory" ? "directory" : "file"}
                    onClick={() =>
                        fileType === "Directory"
                            ? onDirectoryClick(fileName)
                            : undefined
                    }
                    key={idx}
                    name={fileName}
                />
            );
        })}

    </>;
}