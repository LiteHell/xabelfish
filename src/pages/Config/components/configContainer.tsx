import { Button, Flex, Typography } from "antd";
import useXabelFishConfig, {
  EmptyXabelFishConfig,
  XabelFishConfig,
} from "../../../stores/config";
import { useEffect, useState } from "react";
import useApp from "antd/es/app/useApp";
import AppearanceConfigForm from "./appearnceForm";
import TranslationConfigForm from "./translationForm";
import License from "./license";
import { invoke } from "@tauri-apps/api/core";
import { isEqual } from "es-toolkit";
import SaveFooter from "./saveFooter";
import TesseractConfigForm from "./tesseractConfigForm";

export default function ConfigContainer() {
  const config = useXabelFishConfig();
  const antdApp = useApp();
  const [formValue, setFormValue] = useState<XabelFishConfig>({
    ...EmptyXabelFishConfig,
  });
  useEffect(() => {
    setFormValue(config.config);
  }, [config.config]);

  const handleSave = () => {
    config
      .save(formValue)
      .then(() => {
        antdApp.message.success("Success");
      })
      .catch((err) => {
        antdApp.message.error("Error occured: " + err.toString());
      });
  };

  const setWindow = () => {
    invoke("set_window");
  };

  const dirty = !isEqual(formValue, config.config);

  return (
    <div style={{ padding: "20px" }}>
      <Typography.Title level={4}>Window/Region selection</Typography.Title>
      <Flex gap="10px">
        <Button type="primary" onClick={setWindow}>
          Select window or region
        </Button>
      </Flex>

      <Typography.Title level={4}>Config</Typography.Title>

      <Typography.Title level={5}>Translation API</Typography.Title>
      <TranslationConfigForm
        value={formValue}
        onChange={setFormValue}
      ></TranslationConfigForm>

      <Typography.Title level={5}>OCR</Typography.Title>
      <TesseractConfigForm
        value={formValue}
        onChange={setFormValue}
      ></TesseractConfigForm>

      <Typography.Title level={5}>Appearance</Typography.Title>
      <AppearanceConfigForm
        value={formValue}
        fontFamilies={config.fontFamilies}
        onChange={setFormValue}
      ></AppearanceConfigForm>

      <SaveFooter
        isConfigDirty={dirty}
        onSaveConfigClick={handleSave}
        onDiscardChangesClick={() => setFormValue(config.config)}
      />

      <Typography.Title level={4}>License</Typography.Title>
      <License />
    </div>
  );
}
