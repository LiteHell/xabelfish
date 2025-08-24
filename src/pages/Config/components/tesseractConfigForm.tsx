import { Form, Input } from "antd";
import { XabelFishConfig } from "../../../stores/config";
import StyledForm from "./styledForm";

type TesseractConfigFormProp = {
  value: XabelFishConfig;
  onChange: (value: XabelFishConfig) => void;
};

export default function TesseractConfigForm({
  value,
  onChange,
}: TesseractConfigFormProp) {
  return (
    <StyledForm>
      <Form.Item
        label="Tesseract language"
        extra={
          <>
            You need to install tesseract OCR data. Otherwise, It will cause
            error.
          </>
        }
      >
        <Input
          value={value.tesseract_language}
          onChange={(evt) =>
            onChange({
              ...value,
              tesseract_language: evt.target.value,
            })
          }
        ></Input>
      </Form.Item>
    </StyledForm>
  );
}
