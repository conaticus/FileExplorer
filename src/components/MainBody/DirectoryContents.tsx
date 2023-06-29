import DirectoryEntity from "./DirectoryEntity";
import { DirectoryContent } from "../../types";

interface Props {
  content: DirectoryContent[];
  onDirectoryClick: (filePath: string) => any;
}

export function DirectoryContents({ content, onDirectoryClick }: Props) {
  return (
    <div className=" rounded-lg overflow-hidden">
      {content.length === 0 ? "There are no files in this directory." : ""}

      {content.map((content, idx) => {
        const [fileType, [fileName, filePath]] = Object.entries(content)[0];

        return (
          <DirectoryEntity
            type={fileType === "Directory" ? "directory" : "file"}
            onClick={() =>
              fileType === "Directory" ? onDirectoryClick(filePath) : undefined
            }
            key={idx}
            name={fileName}
          />
        );
      })}
    </div>
  );
}
