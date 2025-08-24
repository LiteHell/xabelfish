import { create } from "zustand";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

export type XabelFishConfig = {
  deepl_api_key: string;
  font_family: string;
  font_size: number;
  background_color: string;
  font_color: string;
};

type XabelFishConfigStore = {
  config: XabelFishConfig;
  fontFamilies: string[];
  fontLoaded: boolean;
  fetch: () => Promise<void>;
  __set_data_raw_unsafe: (config: XabelFishConfig) => void;
  save: (newConfig: XabelFishConfig) => Promise<void>;
};

export const EmptyXabelFishConfig: XabelFishConfig = Object.freeze({
  deepl_api_key: "",
  font_family: "sans-serif",
  font_size: 30,
  background_color: "#00000011",
  font_color: "#ffffffff",
});

const useXabelFishConfigStore = create<XabelFishConfigStore>((set, get) => ({
  config: { ...EmptyXabelFishConfig },
  fontFamilies: ["sans-serif", "serif"],
  fontLoaded: false,
  fetch() {
    return new Promise((resolve, reject) => {
      invoke<XabelFishConfig>("get_config")
        .then((config) => set({ config }))
        .then(() => {
          if (!get().fontLoaded) {
            return invoke<string[]>("get_fonts").then((fontFamilies) =>
              set({ fontFamilies, fontLoaded: true })
            );
          } else {
            return Promise.resolve();
          }
        })
        .then(() => resolve())
        .catch(reject);
    });
  },
  __set_data_raw_unsafe(config) {
    set({ config });
  },
  async save(newConfig) {
    await invoke("set_config", { config: newConfig });
    await get().fetch();
  },
}));
export default function useXabelFishConfig() {
  const store = useXabelFishConfigStore();

  listen<XabelFishConfig>("config_changed", (evt) =>
    store.__set_data_raw_unsafe(evt.payload)
  );

  return store;
}
