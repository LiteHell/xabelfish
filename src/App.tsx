import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import "./message.css";
import { listen } from "@tauri-apps/api/event";

const displayLicense = () => {
  alert(`XabelFish - Game translator for Unix-like operating systems
Copyright (C) 2025 Yeonjin Shin (a.k.a. LiteHell)

This program is free software: you can redistribute it and/or modify
it under the terms of the GNU General Public License as published by
the Free Software Foundation, either version 3 of the License, or
(at your option) any later version.

This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU General Public License for more details.

You should have received a copy of the GNU General Public License
along with this program.  If not, see <https://www.gnu.org/licenses/>.`);
};

type XabelFishConfig = {
  deepl_api_key: string;
};

function App() {
  const [translated, setTranslated] = useState<string | null>(null);
  listen<string>("translated", (e) => setTranslated(e.payload));

  const setWindow = () => {
    invoke("set_window");
  };

  const setConfig = () => {
    invoke<XabelFishConfig>("get_config").then((config) => {
      const newDeepLApiKey = prompt(
        "DeepL API Key?\n(NOTE: Settings is development in progress....)",
        config.deepl_api_key
      );
      if (newDeepLApiKey !== null) {
        const newConfig: XabelFishConfig = {
          ...config,
          deepl_api_key: newDeepLApiKey,
        };

        invoke("set_config", { config: newConfig })
          .then(() => alert("Done"))
          .catch((err) => {
            alert("Oops, somethings's wrong");
            console.error(err);
          });
      }
    });
  };

  return (
    <main>
      <nav>
        <ul>
          <li onClick={setWindow}>Select window/area</li>
          <li onClick={setConfig}>Settings</li>
          <li onClick={displayLicense}>License</li>
        </ul>
      </nav>
      <article>{translated ?? "Hello, XabelFish!"}</article>
    </main>
  );
}

export default App;
