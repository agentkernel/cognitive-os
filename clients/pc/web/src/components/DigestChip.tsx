import { useState } from "react";

/** Medial truncation for digests/refs: keep both ends readable. */
export function truncateMiddle(value: string, keep = 6): string {
  if (value.length <= keep * 2 + 1) {
    return value;
  }
  return `${value.slice(0, keep)}…${value.slice(-keep)}`;
}

/**
 * DigestChip — mono digest/ref chip with a copy affordance. Copies the full
 * value, displays the truncated one. No secret-shaped value may ever be
 * passed here (callers enforce; the redaction boundary enforces upstream).
 */
export function DigestChip({ value, label }: { value: string; label?: string }) {
  const [copied, setCopied] = useState(false);
  return (
    <span className="cp-digestchip" title={value}>
      {label ? <span className="cp-quiet">{label}</span> : null}
      <span>{truncateMiddle(value)}</span>
      <button
        type="button"
        aria-label={`copy ${label ?? "digest"}`}
        onClick={() => {
          void navigator.clipboard.writeText(value).then(() => {
            setCopied(true);
            setTimeout(() => setCopied(false), 1200);
          });
        }}
      >
        {copied ? "copied" : "copy"}
      </button>
    </span>
  );
}
