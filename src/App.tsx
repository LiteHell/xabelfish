import { BrowserRouter, Route, Routes } from "react-router";
import TranslationOverlay from "./pages/TranslationOverlay";
import Config from "./pages/Config";
import { useEffect } from "react";
import useXabelFishConfig from "./stores/config";

function App() {
  // Fetches config at initialization
  const config = useXabelFishConfig();
  useEffect(() => {
    console.log("fetched");
    config.fetch().then(() => console.log(config.config));
  }, []);

  return (
    <Routes>
      <Route path="/" element={<TranslationOverlay />} />
      <Route path="/config" element={<Config />} />
    </Routes>
  );
}

export default App;
