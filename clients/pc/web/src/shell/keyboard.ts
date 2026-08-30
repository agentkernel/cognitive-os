/**
 * Shell keyboard — docs/design/12. Chords and list motion observe the current
 * document; they never mint authority.
 *
 * `g` then t/p/n/s jumps Today/Projects/Knowledge/Settings; w/a/h/v/r/c keep
 * Work/Agents/Home/Providers/Resources/Activity. `/` opens the palette.
 * `j`/`k` move the selected master row. Enter inspects, then opens detail.
 * `[`/`]` walk Work-detail sections, or the master list when there is no
 * section navigator. Escape unwinds one layer: palette (its own handler) →
 * detail back-link → inspector selection.
 */

export const SPACE_CHORDS: Record<string, string> = {
  t: "/",
  p: "/projects",
  n: "/knowledge",
  s: "/settings",
  h: "/home",
  w: "/work",
  a: "/agents",
  v: "/providers",
  r: "/resources",
  c: "/activity",
};

export const G_CHORD_MS = 800;

type InspectorClear = () => boolean;

let inspectorClear: InspectorClear | null = null;

export function registerInspectorClear(fn: InspectorClear): () => void {
  inspectorClear = fn;
  return () => {
    if (inspectorClear === fn) {
      inspectorClear = null;
    }
  };
}

export function tryClearInspector(): boolean {
  return inspectorClear?.() ?? false;
}

export function isTypingTarget(target: EventTarget | null): boolean {
  if (!(target instanceof HTMLElement)) {
    return false;
  }
  const tag = target.tagName;
  if (tag === "INPUT" || tag === "TEXTAREA" || tag === "SELECT") {
    return true;
  }
  return target.isContentEditable === true;
}

export function isEnterReserved(target: EventTarget | null): boolean {
  if (!(target instanceof HTMLElement)) {
    return false;
  }
  if (isTypingTarget(target)) {
    return true;
  }
  const tag = target.tagName;
  return tag === "BUTTON" || tag === "A" || tag === "SUMMARY";
}

function scopedMain(root: ParentNode): ParentNode {
  if (root instanceof Element && root.matches("main")) {
    return root;
  }
  return root.querySelector("main") ?? root;
}

export function masterRows(root: ParentNode = document): HTMLElement[] {
  const scope = scopedMain(root);
  const tableRows = [...scope.querySelectorAll<HTMLElement>("tr[data-row-key]")];
  if (tableRows.length > 0) {
    return tableRows;
  }
  return [...scope.querySelectorAll<HTMLElement>("li.cp-queue-row")].filter((row) =>
    inspectControl(row),
  );
}

function inspectControl(row: HTMLElement): HTMLElement | undefined {
  const buttons = [...row.querySelectorAll("button")];
  return (
    buttons.find((button) => (button.textContent ?? "").trim() === "Inspect") ?? buttons[0]
  );
}

function openControl(row: HTMLElement): HTMLElement | undefined {
  return [...row.querySelectorAll("a")].find((node) => (node.textContent ?? "").trim() === "Open");
}

export function stepMaster(direction: 1 | -1, root: ParentNode = document): boolean {
  const rows = masterRows(root);
  if (rows.length === 0) {
    return false;
  }
  const current = rows.findIndex((row) => row.getAttribute("aria-selected") === "true");
  const next = Math.max(0, Math.min(rows.length - 1, (current < 0 ? 0 : current) + direction));
  inspectControl(rows[next])?.click();
  rows[next]?.scrollIntoView?.({ block: "nearest" });
  return true;
}

/** First Enter inspects; Enter on an already-selected row opens detail. */
export function openSelectedMaster(root: ParentNode = document): boolean {
  const rows = masterRows(root);
  if (rows.length === 0) {
    return false;
  }
  const selected =
    rows.find((row) => row.getAttribute("aria-selected") === "true") ?? undefined;
  if (!selected) {
    inspectControl(rows[0])?.click();
    return true;
  }
  const open = openControl(selected);
  if (open) {
    open.click();
    return true;
  }
  inspectControl(selected)?.click();
  return true;
}

export function stepDetailSection(direction: 1 | -1, root: ParentNode = document): boolean {
  const links = [...root.querySelectorAll<HTMLButtonElement>(".cp-sectionnav-link")];
  if (links.length === 0) {
    return false;
  }
  const current = links.findIndex((link) => link.getAttribute("aria-current") === "true");
  const next = Math.max(0, Math.min(links.length - 1, (current < 0 ? 0 : current) + direction));
  links[next]?.click();
  return true;
}

export function findBackLink(root: ParentNode = document): HTMLAnchorElement | undefined {
  return [...root.querySelectorAll("a")].find((node) =>
    /^Back to /.test((node.textContent ?? "").trim()),
  );
}

export function unwindDetail(root: ParentNode = document): boolean {
  const back = findBackLink(root);
  if (!back) {
    return false;
  }
  back.click();
  return true;
}
