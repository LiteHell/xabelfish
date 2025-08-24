import React, { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import "./message.css";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWebview } from "@tauri-apps/api/webview";
import useXabelFishConfig from "../../stores/config";

function TranslationOverlay() {
  const appWindow = getCurrentWebview();
  const { config } = useXabelFishConfig();
  const [translated, setTranslated] = useState<string | null>(null);
  listen<string>("translated", (e) => setTranslated(e.payload));

  const dragOrOpenConfig: React.MouseEventHandler<HTMLElement> = (evt) => {
    if (evt.button === 2) {
      evt.preventDefault();
      invoke("open_config_window");
    } else {
      appWindow.window.startDragging();
    }
  };

  return (
    <main
      style={{
        backgroundColor: config.background_color,
        color: config.font_color,
        fontFamily: config.font_family,
        fontSize: config.font_size + "pt",
      }}
      onContextMenu={(evt) => evt.preventDefault()}
    >
      <article onMouseDown={dragOrOpenConfig}>
        {translated ?? "Hello, XabelFish! Right-click to open the settings"}
      </article>
    </main>
  );
}

export default TranslationOverlay;
