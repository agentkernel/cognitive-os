/**
 * Canonical JSON (`cognitiveos.canonical-json/0.1`) and domain-separated
 * digest / signature preimages — TypeScript twin of
 * `crates/cognitive-contracts/src/canonical.rs`.
 *
 * Implements `docs/standards/canonical-encoding-and-digest.md` sections 2-9
 * and 12 for the encoding layer: UTF-8 without BOM, I-JSON, RFC 8785 JCS
 * output, duplicate-member rejection, unsafe-integer rejection, and the
 * `CognitiveOS-Digest-V1` / `CognitiveOS-Signature-V1` preimages.
 *
 * RFC 8785 note: JCS string escaping and number serialization are defined in
 * terms of ECMAScript `JSON.stringify` / `ToString(Number)`, so the native
 * primitives are used directly; ordering is by UTF-16 code units, which is
 * plain JavaScript string comparison.
 */

import { createHash } from "node:crypto";

/** I-JSON safe integer bound (RFC 7493): 2^53 - 1. */
export const MAX_SAFE_INTEGER_LITERAL = "9007199254740991";

/** Exact ASCII prefix of every digest preimage (standard section 9). */
export const DIGEST_PREIMAGE_PREFIX = "CognitiveOS-Digest-V1\n";

/** Exact ASCII prefix of every signature input (standard section 12). */
export const SIGNATURE_PREIMAGE_PREFIX = "CognitiveOS-Signature-V1\n";

/**
 * Rejection categories shared with the Rust implementation and the golden
 * fixtures under `tests/golden/`. String forms are fixture contract.
 */
export type CanonicalErrorCategory =
  | "invalid-utf8"
  | "bom"
  | "duplicate-member-name"
  | "unsafe-integer"
  | "invalid-json"
  | "invalid-domain"
  | "invalid-algorithm"
  | "canonicalization-failed";

export class CanonicalError extends Error {
  readonly category: CanonicalErrorCategory;

  constructor(category: CanonicalErrorCategory, detail: string) {
    super(detail ? `${category}: ${detail}` : category);
    this.name = "CanonicalError";
    this.category = category;
  }
}

/** Strict JSON value. Objects preserve document member order. */
export type StrictValue =
  | null
  | boolean
  | number
  | string
  | StrictValue[]
  | { readonly kind: "object"; readonly members: ReadonlyArray<readonly [string, StrictValue]> };

function isObjectValue(
  value: StrictValue,
): value is { readonly kind: "object"; readonly members: ReadonlyArray<readonly [string, StrictValue]> } {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

/**
 * Strict recursive-descent JSON parser (standard section 3): rejects BOM,
 * duplicate member names, unsafe integer literals, unpaired surrogate
 * escapes and every JSON grammar violation. `JSON.parse` cannot be used
 * because it silently keeps the last duplicate member and widens integers.
 */
class StrictParser {
  private pos = 0;

  constructor(private readonly text: string) {}

  parse(): StrictValue {
    this.skipWs();
    const value = this.parseValue();
    this.skipWs();
    if (this.pos !== this.text.length) {
      throw new CanonicalError("invalid-json", `trailing characters at offset ${this.pos}`);
    }
    return value;
  }

  private fail(reason: string): never {
    throw new CanonicalError("invalid-json", `${reason} at offset ${this.pos}`);
  }

  private skipWs(): void {
    while (this.pos < this.text.length) {
      const c = this.text[this.pos];
      if (c === " " || c === "\t" || c === "\n" || c === "\r") {
        this.pos += 1;
      } else {
        break;
      }
    }
  }

  private parseValue(): StrictValue {
    const c = this.text[this.pos];
    if (c === undefined) {
      this.fail("unexpected end of input");
    }
    switch (c) {
      case "{":
        return this.parseObject();
      case "[":
        return this.parseArray();
      case '"':
        return this.parseString();
      case "t":
        this.expectLiteral("true");
        return true;
      case "f":
        this.expectLiteral("false");
        return false;
      case "n":
        this.expectLiteral("null");
        return null;
      default:
        if (c === "-" || (c >= "0" && c <= "9")) {
          return this.parseNumber();
        }
        return this.fail(`unexpected character ${JSON.stringify(c)}`);
    }
  }

  private expectLiteral(literal: string): void {
    if (this.text.startsWith(literal, this.pos)) {
      this.pos += literal.length;
    } else {
      this.fail(`invalid literal, expected ${literal}`);
    }
  }

  private parseObject(): StrictValue {
    this.pos += 1; // consume '{'
    const members: Array<readonly [string, StrictValue]> = [];
    const seen = new Set<string>();
    this.skipWs();
    if (this.text[this.pos] === "}") {
      this.pos += 1;
      return { kind: "object", members };
    }
    for (;;) {
      this.skipWs();
      if (this.text[this.pos] !== '"') {
        this.fail("expected member name string");
      }
      const name = this.parseString();
      if (seen.has(name)) {
        throw new CanonicalError("duplicate-member-name", name);
      }
      seen.add(name);
      this.skipWs();
      if (this.text[this.pos] !== ":") {
        this.fail("expected ':' after member name");
      }
      this.pos += 1;
      this.skipWs();
      members.push([name, this.parseValue()] as const);
      this.skipWs();
      const next = this.text[this.pos];
      if (next === ",") {
        this.pos += 1;
        continue;
      }
      if (next === "}") {
        this.pos += 1;
        return { kind: "object", members };
      }
      this.fail("expected ',' or '}' in object");
    }
  }

  private parseArray(): StrictValue {
    this.pos += 1; // consume '['
    const items: StrictValue[] = [];
    this.skipWs();
    if (this.text[this.pos] === "]") {
      this.pos += 1;
      return items;
    }
    for (;;) {
      this.skipWs();
      items.push(this.parseValue());
      this.skipWs();
      const next = this.text[this.pos];
      if (next === ",") {
        this.pos += 1;
        continue;
      }
      if (next === "]") {
        this.pos += 1;
        return items;
      }
      this.fail("expected ',' or ']' in array");
    }
  }

  private parseString(): string {
    this.pos += 1; // consume opening quote
    let out = "";
    for (;;) {
      const c = this.text[this.pos];
      if (c === undefined) {
        this.fail("unterminated string");
      }
      if (c === '"') {
        this.pos += 1;
        return out;
      }
      if (c === "\\") {
        out += this.parseEscape();
        continue;
      }
      const code = c.codePointAt(0) ?? 0;
      if (code < 0x20) {
        this.fail("unescaped control character in string");
      }
      // Unpaired surrogates cannot survive UTF-8 decoding with fatal=true,
      // but guard against direct string inputs.
      if (code >= 0xd800 && code <= 0xdfff) {
        const pair = this.text.codePointAt(this.pos) ?? 0;
        if (pair >= 0xd800 && pair <= 0xdfff) {
          this.fail("unpaired surrogate in string");
        }
        const composed = String.fromCodePoint(pair);
        out += composed;
        this.pos += composed.length;
        continue;
      }
      out += c;
      this.pos += 1;
    }
  }

  private parseEscape(): string {
    this.pos += 1; // consume backslash
    const c = this.text[this.pos];
    this.pos += 1;
    switch (c) {
      case '"':
        return '"';
      case "\\":
        return "\\";
      case "/":
        return "/";
      case "b":
        return "\b";
      case "f":
        return "\f";
      case "n":
        return "\n";
      case "r":
        return "\r";
      case "t":
        return "\t";
      case "u": {
        const unit = this.parseHex4();
        if (unit >= 0xd800 && unit <= 0xdbff) {
          // Expect a low-surrogate escape to complete the pair.
          if (this.text[this.pos] === "\\" && this.text[this.pos + 1] === "u") {
            this.pos += 2;
            const low = this.parseHex4();
            if (low >= 0xdc00 && low <= 0xdfff) {
              return String.fromCharCode(unit, low);
            }
          }
          this.fail("unpaired surrogate escape");
        }
        if (unit >= 0xdc00 && unit <= 0xdfff) {
          this.fail("unpaired surrogate escape");
        }
        return String.fromCharCode(unit);
      }
      default:
        return this.fail("invalid escape sequence");
    }
  }

  private parseHex4(): number {
    const hex = this.text.slice(this.pos, this.pos + 4);
    if (!/^[0-9a-fA-F]{4}$/.test(hex)) {
      this.fail("invalid \\u escape");
    }
    this.pos += 4;
    return Number.parseInt(hex, 16);
  }

  private parseNumber(): number {
    const start = this.pos;
    const rest = this.text.slice(start);
    const match = /^-?(?:0|[1-9][0-9]*)(?:\.[0-9]+)?(?:[eE][+-]?[0-9]+)?/.exec(rest);
    if (!match || match[0].length === 0) {
      this.fail("invalid number literal");
    }
    const literal = match[0];
    this.pos += literal.length;
    const isIntegerForm = !/[.eE]/.test(literal);
    if (isIntegerForm) {
      const digits = literal.startsWith("-") ? literal.slice(1) : literal;
      const tooBig =
        digits.length > MAX_SAFE_INTEGER_LITERAL.length ||
        (digits.length === MAX_SAFE_INTEGER_LITERAL.length && digits > MAX_SAFE_INTEGER_LITERAL);
      if (tooBig) {
        throw new CanonicalError("unsafe-integer", literal);
      }
    }
    const value = Number(literal);
    if (!Number.isFinite(value)) {
      throw new CanonicalError("unsafe-integer", literal);
    }
    return value;
  }
}

/** Strictly parse a JSON text (standard sections 3-5). */
export function parseStrict(input: string): StrictValue {
  if (input.startsWith("\uFEFF")) {
    throw new CanonicalError("bom", "input begins with a byte order mark");
  }
  return new StrictParser(input).parse();
}

/** Strictly decode UTF-8 bytes, then parse. Invalid UTF-8 and BOM are rejected. */
export function parseStrictBytes(input: Uint8Array): StrictValue {
  let text: string;
  try {
    text = new TextDecoder("utf-8", { fatal: true, ignoreBOM: true }).decode(input);
  } catch {
    throw new CanonicalError("invalid-utf8", "input is not well-formed UTF-8");
  }
  return parseStrict(text);
}

function serializeCanonical(value: StrictValue): string {
  if (value === null) {
    return "null";
  }
  switch (typeof value) {
    case "boolean":
      return value ? "true" : "false";
    case "number": {
      if (!Number.isFinite(value)) {
        throw new CanonicalError("canonicalization-failed", "non-finite number");
      }
      // RFC 8785: ECMAScript ToString(Number); ToString(-0) is "0".
      return Object.is(value, -0) ? "0" : String(value);
    }
    case "string":
      // RFC 8785 string serialization is ECMAScript JSON.stringify quoting.
      return JSON.stringify(value);
    default:
      break;
  }
  if (Array.isArray(value)) {
    return `[${value.map(serializeCanonical).join(",")}]`;
  }
  if (isObjectValue(value)) {
    // JCS: sort member names by UTF-16 code units (plain JS comparison).
    const sorted = [...value.members].sort(([a], [b]) => (a < b ? -1 : a > b ? 1 : 0));
    const body = sorted
      .map(([name, member]) => `${JSON.stringify(name)}:${serializeCanonical(member)}`)
      .join(",");
    return `{${body}}`;
  }
  throw new CanonicalError("canonicalization-failed", "unsupported value");
}

/** Produce RFC 8785 canonical UTF-8 bytes for a strict-parsed value. */
export function canonicalBytes(value: StrictValue): Uint8Array {
  return new TextEncoder().encode(serializeCanonical(value));
}

/** Strict-parse a JSON text and return its RFC 8785 canonical bytes. */
export function canonicalize(input: string): Uint8Array {
  return canonicalBytes(parseStrict(input));
}

const DOMAIN_GRAMMAR = /^[a-z0-9][a-z0-9._/-]{0,127}$/;
const FORBIDDEN_DOMAINS = new Set(["generic", "object", "payload"]);

/** Validate a digest/signature domain label (standard section 9). */
export function validateDomain(domain: string): void {
  if (!DOMAIN_GRAMMAR.test(domain) || FORBIDDEN_DOMAINS.has(domain)) {
    throw new CanonicalError("invalid-domain", domain);
  }
}

function concatBytes(parts: ReadonlyArray<Uint8Array>): Uint8Array {
  const total = parts.reduce((sum, part) => sum + part.length, 0);
  const out = new Uint8Array(total);
  let offset = 0;
  for (const part of parts) {
    out.set(part, offset);
    offset += part.length;
  }
  return out;
}

const ascii = (text: string): Uint8Array => new TextEncoder().encode(text);

/** Exact digest preimage: `"CognitiveOS-Digest-V1\n" || domain || 0x00 || C`. */
export function digestPreimage(canonical: Uint8Array, domain: string): Uint8Array {
  validateDomain(domain);
  return concatBytes([ascii(DIGEST_PREIMAGE_PREFIX), ascii(domain), Uint8Array.of(0), canonical]);
}

/** Domain-separated digest: `"sha256:" || lowercase_hex(SHA-256(preimage))`. */
export function digest(canonical: Uint8Array, domain: string): string {
  const hash = createHash("sha256").update(digestPreimage(canonical, domain)).digest("hex");
  return `sha256:${hash}`;
}

/**
 * Exact signature input (standard section 12):
 * `"CognitiveOS-Signature-V1\n" || domain || 0x00 || algorithm || 0x00 || C`.
 */
export function signatureInput(canonical: Uint8Array, domain: string, algorithm: string): Uint8Array {
  validateDomain(domain);
  if (algorithm.length === 0 || !/^[\x00-\x7F]+$/.test(algorithm)) {
    throw new CanonicalError("invalid-algorithm", algorithm);
  }
  return concatBytes([
    ascii(SIGNATURE_PREIMAGE_PREFIX),
    ascii(domain),
    Uint8Array.of(0),
    ascii(algorithm),
    Uint8Array.of(0),
    canonical,
  ]);
}

/** Convenience: strict parse + canonicalize + domain-separated digest. */
export function digestJson(input: string, domain: string): string {
  return digest(canonicalize(input), domain);
}
