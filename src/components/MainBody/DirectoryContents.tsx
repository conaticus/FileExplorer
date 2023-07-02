import DirectoryEntity from "./DirectoryEntity";
import {DirectoryContent, DirectoryContentType} from "../../types";
import {openFile} from "../../ipc";

interface Props {
    content: DirectoryContent[];
    onDirectoryClick: (filePath: string) => any;
}

export function DirectoryContents({content, onDirectoryClick}: Props) {
    async function onFileClick(path: string) {
        await openFile(path).catch(err => alert(err));
    }

    return <>
        {content.length === 0 ? "There are no files in this directory." : ""}

        {content.map((content, idx) => {
            const [fileType, [fileName, filePath]] = Object.entries(content)[0];

            return (
                <DirectoryEntity
                    type={fileType as DirectoryContentType}
                    onDoubleClick={() =>
                        fileType === "Directory"
                            ? onDirectoryClick(filePath)
                            : onFileClick(filePath)
                    }
                    key={idx}
                    idx={idx}
                    name={fileName}
                    path={filePath}
                />
            );
        })}

    </>;
}