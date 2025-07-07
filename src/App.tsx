import { useState } from "react";
import reactLogo from "./assets/react.svg";
import { invoke } from "@tauri-apps/api/core";
import "./message.css";
import { listen } from "@tauri-apps/api/event";

function App() {
  const [translated, setTranslated] = useState<string | null>(null);
  listen<string>("translated", (e) => setTranslated(e.payload));
  
  const setWindow = () => {
    invoke('set_window');
  }

  return (
    <main>
      <nav>
        <ul>
          <li onClick={setWindow}>Select window/area</li>
          <li>Settings</li>
        </ul>
      </nav>
      <article>
        { translated ?? 'Hello, Lugata!' }
      </article>
    </main>
  );
}

export default App;
