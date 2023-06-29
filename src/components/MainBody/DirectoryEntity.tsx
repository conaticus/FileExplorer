import { MouseEventHandler } from "react";
import { DirectoryEntityType } from "../../types";
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome";
import { faFile, faFolder } from "@fortawesome/free-solid-svg-icons";

interface Props {
  name: string;
  type: DirectoryEntityType;
  onClick?: MouseEventHandler<HTMLButtonElement>;
}

export default function DirectoryEntity({ name, type, onClick }: Props) {
  return (
    <>
      <button
        className="bg-background hover:bg-darker cursor-pointer w-full h-[100%] flex p-3"
        onClick={onClick}
      >
        <div className="mr-1">
          <FontAwesomeIcon
            icon={type == "file" ? faFile : faFolder}
            size="lg"
            color={type == "file" ? "gray" : "#FFD54F"}
          />
        </div>
        {name}
      </button>
    </>
  );
}
