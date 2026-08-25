import { describe, expect, it } from "vitest";
import {
  PROVIDER_KINDS,
  capabilityDisposition,
  classifyProbe,
  requiresTrustConfirmation,
} from "./probe";

describe("provider kinds", () => {
  it("uses daemon tokens, not openai/anthropic aliases", () => {
    expect(PROVIDER_KINDS).toEqual([
      "openai_official",
      "anthropic_official",
      "openai_compatible",
    ]);
  });
});

describe("trust confirmation", () => {
  it("requires an extra confirmation only for custom private or HTTP endpoints", () => {
    expect(
      requiresTrustConfirmation({
        kind: "openai_official",
        allowPrivateNetwork: true,
        allowInsecureHttp: true,
      }),
    ).toBe(false);
    expect(
      requiresTrustConfirmation({
        kind: "openai_compatible",
        allowPrivateNetwork: true,
        allowInsecureHttp: false,
      }),
    ).toBe(true);
    expect(
      requiresTrustConfirmation({
        kind: "openai_compatible",
        allowPrivateNetwork: false,
        allowInsecureHttp: false,
      }),
    ).toBe(false);
  });
});

describe("probe classes", () => {
  it("distinguishes authentication, reachability, and discovery", () => {
    expect(
      classifyProbe({
        ok: false,
        httpStatus: 409,
        body: { code: "PROVIDER_KEY_MISSING" },
      }).class,
    ).toBe("authentication");
    expect(
      classifyProbe({
        ok: false,
        httpStatus: 502,
        body: { code: "PROVIDER_DISCOVERY_FAILED", message: "discovery transport failed" },
      }).class,
    ).toBe("reachability");
    expect(
      classifyProbe({
        ok: false,
        httpStatus: 502,
        body: { code: "PROVIDER_DISCOVERY_MALFORMED" },
      }).class,
    ).toBe("model_discovery");
    expect(
      classifyProbe({
        ok: true,
        httpStatus: 200,
        body: { status: "ok", models: [] },
      }).label,
    ).toBe("model_discovery");
  });

  it("does not treat missing capability flags as ready", () => {
    expect(capabilityDisposition(undefined)).toBe("not-run");
    expect(capabilityDisposition(null)).toBe("not-run");
  });
});
