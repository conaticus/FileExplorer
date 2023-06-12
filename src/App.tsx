import {useEffect, useState} from "react";
import reactLogo from "./assets/react.svg";
import { invoke } from "@tauri-apps/api/tauri";

interface Disk {
    name: string;
    used_gb: number;
    total_gb: number;
}

function App() {
    const [disks, setDisks] = useState<Disk[]>([]);
    useEffect(() => {
        async function getData() {
            const disks = await invoke<Disk[]>("get_disks");
            setDisks(disks);
        }

        getData().catch(console.error);
    }, [])

    console.log(disks);

  return (
      <div></div>
  );
}

export default App;
