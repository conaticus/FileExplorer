import {useEffect} from "react";
import { invoke } from "@tauri-apps/api/tauri";

interface Disk {
    name: string;
    used_gb: number;
    total_gb: number;
}

interface DiskContents {
    [key: string]: string; // Key will be either "Directory" or "File"
}

function App() {
    useEffect(() => {
        async function getData() {
            const disks = await invoke<Disk[]>("get_disks");
            console.log(disks);

            const diskContents = await invoke("open_disk", { diskLetter: "C" });
            console.log(diskContents)
        }

        getData().catch(console.error);
    }, [])

  return (
      <div></div>
  );
}

export default App;
