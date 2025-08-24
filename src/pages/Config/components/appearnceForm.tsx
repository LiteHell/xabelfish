import { ColorPicker, Form, InputNumber, Select } from "antd";
import { XabelFishConfig } from "../../../stores/config";
import StyledForm from "./styledForm";

type AppearanceConfigFormProp = {
  value: XabelFishConfig;
  onChange: (value: XabelFishConfig) => void;
  fontFamilies: string[];
};

export default function AppearanceConfigForm({
  value,
  fontFamilies,
  onChange,
}: AppearanceConfigFormProp) {
  return (
    <StyledForm>
      <Form.Item label="Text Color">
        <ColorPicker
          value={value.font_color}
          onChange={(color) =>
            onChange({
              ...value,
              font_color: color.toHexString(),
            })
          }
        ></ColorPicker>
      </Form.Item>
      <Form.Item label="Font">
        <Select
          value={value.font_family}
          onChange={(font_family) =>
            onChange({
              ...value,
              font_family,
            })
          }
        >
          {fontFamilies.map((name) => (
            <Select.Option value={name}>{name}</Select.Option>
          ))}
          <Select.Option value="sans-serif">
            System default sans-serif font
          </Select.Option>
          <Select.Option value="serif">System default serif font</Select.Option>
        </Select>
      </Form.Item>
      <Form.Item label="Text size">
        <InputNumber
          addonAfter="pt"
          value={value.font_size}
          onChange={(font_size) => {
            if (font_size) {
              onChange({ ...value, font_size });
            }
          }}
        ></InputNumber>
      </Form.Item>
      <Form.Item label="Background Color">
        <ColorPicker
          value={value.background_color}
          onChange={(color) =>
            onChange({
              ...value,
              background_color: color.toHexString(),
            })
          }
        ></ColorPicker>
      </Form.Item>
    </StyledForm>
  );
}
