import { faHome, faPlay, faPlayCircle } from "@fortawesome/free-solid-svg-icons";
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome";

interface Props {
    currentDirectoryPath: string;
    onDirectoryClick: Function;
}

export default function FilePathBar({
    currentDirectoryPath,
    onDirectoryClick,
}:Props){
    async function onBarSectionClick(clickedDirectory:string){
        if (clickedDirectory === '') {
            onDirectoryClick(clickedDirectory);
        }

        const clickedSubstringStartIndex = currentDirectoryPath.indexOf(clickedDirectory);
        const fullDirectoryEndIndex = clickedSubstringStartIndex + clickedDirectory.length;
        
        const clickedDirectoryFullPath = currentDirectoryPath.slice(0, fullDirectoryEndIndex);
        onDirectoryClick(clickedDirectoryFullPath);
    }

    return (
        <div className="flex mr-2 p-1 h-10 flex-grow rounded-md border-2 bg-gray-900">
            <button className="pl-1 pr-2 hover:bg-bright" onClick={()=>{onBarSectionClick("")}}>
                <FontAwesomeIcon
                    icon={faHome}
                />
            </button>
            {
            currentDirectoryPath.split("\\").map((subString)=>{
                if (subString === "") return;

                return (
                    <div className="flex">
                        <div className="p-0.5 px-1.5">
                            <FontAwesomeIcon
                                icon={faPlay}
                            />
                        </div>
                        <button className="hover:bg-bright" onClick={()=>{onBarSectionClick(subString)}}>
                            {subString}
                        </button>
                    </div>
                )
            })}
        </div>
    );
}