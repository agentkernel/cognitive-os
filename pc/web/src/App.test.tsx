import { createRoot } from "react-dom/client";
import { act } from "react";
import { describe, expect, it } from "vitest";
import { redactSecrets } from "./policy";
import { exportClientState } from "./session";

describe("DOM and export redaction", () => {
  it("never writes api_key or SecretRef values into the document", () => {
    const host = document.createElement("div");
    document.body.appendChild(host);
    const root = createRoot(host);
    const projection = redactSecrets({
      id: "acct-1",
      api_key: "sk-live-secret",
      secret_ref: "ss://provider/acct-1",
    });
    act(() => {
      root.render(<pre>{JSON.stringify(projection)}</pre>);
    });
    expect(host.textContent).not.toMatch(/sk-live|ss:\/\//);
    expect(host.textContent).toMatch(/"api_key":"present"/);
    expect(exportClientState()).toEqual({});
    act(() => {
      root.unmount();
    });
    host.remove();
  });
});
