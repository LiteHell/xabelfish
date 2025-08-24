import { Form, Input } from "antd";
import { XabelFishConfig } from "../../../stores/config";
import StyledForm from "./styledForm";

type TranslationConfigFormProp = {
  value: XabelFishConfig;
  onChange: (value: XabelFishConfig) => void;
};

export default function TranslationConfigForm({
  value,
  onChange,
}: TranslationConfigFormProp) {
  return (
    <StyledForm>
      <Form.Item
        label="DeepL API Key"
        extra={
          <>
            You can get a API key from{" "}
            <a href="https://www.deepl.com" target="_blank">
              DeepL
            </a>{" "}
            website.
          </>
        }
      >
        <Input
          value={value.deepl_api_key}
          onChange={(evt) =>
            onChange({
              ...value,
              deepl_api_key: evt.target.value,
            })
          }
        ></Input>
      </Form.Item>
    </StyledForm>
  );
}
