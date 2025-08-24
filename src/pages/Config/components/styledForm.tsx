import { Form } from "antd";
import React from "react";

export default function StyledForm({ children }: React.PropsWithChildren<{}>) {
  return (
    <Form labelCol={{ span: 4 }} wrapperCol={{ span: 20 }}>
      {children}
    </Form>
  );
}
