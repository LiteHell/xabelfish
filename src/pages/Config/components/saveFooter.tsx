import { Button, Flex } from "antd";

type SaveFooterProp = {
  onSaveConfigClick: () => void;
  onDiscardChangesClick: () => void;
  isConfigDirty: boolean;
};

export default function SaveFooter({
  onSaveConfigClick,
  onDiscardChangesClick,
  isConfigDirty: isConfigDirty,
}: SaveFooterProp) {
  return (
    <div
      style={{
        padding: "10px",
        position: "fixed",
        width: "100%",
        bottom: "0px",
        left: "0px",
        boxShadow: "rgba(149, 157, 165, 0.2) 0px 8px 24px",
        background: "white",
        boxSizing: "border-box",
      }}
    >
      <Flex justify="flex-end" gap="10px">
        {isConfigDirty ? (
          <Button onClick={onDiscardChangesClick}>Discard changes</Button>
        ) : null}
        <Button
          type="primary"
          onClick={onSaveConfigClick}
          disabled={!isConfigDirty}
        >
          Save config
        </Button>
      </Flex>
    </div>
  );
}
