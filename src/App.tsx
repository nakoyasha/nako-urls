import reactLogo from "./assets/react.svg";
import { invoke } from "@tauri-apps/api/tauri";
import { open } from "@tauri-apps/api/dialog";
import "./App.css";

function App() {
  async function openFile() {
    console.log("a");
    const selected = await open({
      filters: [
        {
          name: "JSON files",
          extensions: ["json"],
        },
      ],
    });
    console.log(selected);

    if (selected == null) {
      alert("pick a file silly");
    } else {
      await invoke("download_urls", {
        filePath: selected,
      });
    }
  }

  return (
    <div className="container">
      <h1>Welcome to Tauri!</h1>

      <div className="row">
        <a href="https://vitejs.dev" target="_blank">
          <img src="/vite.svg" className="logo vite" alt="Vite logo" />
        </a>
        <a href="https://tauri.app" target="_blank">
          <img src="/tauri.svg" className="logo tauri" alt="Tauri logo" />
        </a>
        <a href="https://reactjs.org" target="_blank">
          <img src={reactLogo} className="logo react" alt="React logo" />
        </a>
      </div>

      <p>Click on the Tauri, Vite, and React logos to learn more.</p>
      <button onClick={openFile}>Pick a file to download</button>
    </div>
  );
}

export default App;
