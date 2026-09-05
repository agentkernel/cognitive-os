import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { useLocation, useNavigate } from "react-router-dom";
import { readJson } from "../api";
import { asRecord } from "../data/projections";
import {
  COMMAND_INDEX_HONESTY,
  COMMAND_NO_CLASS_C,
  COMMAND_NO_RESULTS,
  buildCommandCatalog,
  groupCommands,
  rankCommands,
  rememberCommand,
  type CommandItem,
} from "../data/commands";
import { fetchProjection } from "../data/fetchProjection";
import { recordSessionMutation } from "../data/projections/home";
import { projectProviderAlerts } from "../data/projections/providers";
import { appProjections } from "../data/store";
import { HonestyNote } from "../state/HonestyNote";
import { ReceiptLine } from "./ReceiptLine";

/**
 * Command palette — docs/design/21. Speed layer, not a space. Class-A lands
 * on governed routes; class-B runs inline with a receipt. Class-C is absent.
 */
export function CommandPalette({
  open,
  onClose,
}: {
  open: boolean;
  onClose: () => void;
}) {
  const navigate = useNavigate();
  const location = useLocation();
  const inputRef = useRef<HTMLInputElement>(null);
  const dialogRef = useRef<HTMLDivElement>(null);
  const returnFocus = useRef<HTMLElement | null>(null);
  const [query, setQuery] = useState("");
  const [cursor, setCursor] = useState(0);
  const [receipt, setReceipt] = useState<string | undefined>();
  const [message, setMessage] = useState<string | undefined>();
  const [catalogTick, setCatalogTick] = useState(0);

  useEffect(() => appProjections.subscribe(() => setCatalogTick((tick) => tick + 1)), []);

  const groups = useMemo(() => {
    const ranked = rankCommands(
      buildCommandCatalog(appProjections, `${location.pathname}${location.search}`),
      query,
    );
    return groupCommands(ranked, query);
  }, [query, open, catalogTick, location.pathname, location.search]);
  const items = useMemo(() => groups.flatMap((group) => group.items), [groups]);

  const close = useCallback(() => {
    onClose();
    returnFocus.current?.focus();
  }, [onClose]);

  useEffect(() => {
    if (!open) {
      return;
    }
    returnFocus.current =
      document.activeElement instanceof HTMLElement ? document.activeElement : null;
    setQuery("");
    setCursor(0);
    setReceipt(undefined);
    setMessage(undefined);
    const timer = window.setTimeout(() => inputRef.current?.focus(), 0);
    return () => window.clearTimeout(timer);
  }, [open]);

  useEffect(() => {
    setCursor(0);
  }, [query]);

  useEffect(() => {
    if (!open) {
      return;
    }
    function onKey(event: KeyboardEvent) {
      if (event.key === "Escape") {
        event.preventDefault();
        event.stopImmediatePropagation();
        close();
      }
    }
    window.addEventListener("keydown", onKey, true);
    return () => window.removeEventListener("keydown", onKey, true);
  }, [open, close]);

  useEffect(() => {
    if (!open) {
      return;
    }
    function trap(event: KeyboardEvent) {
      if (event.key !== "Tab") {
        return;
      }
      const root = dialogRef.current;
      if (!root) {
        return;
      }
      const focusable = [inputRef.current].filter((node): node is HTMLInputElement => node != null);
      if (focusable.length === 0) {
        return;
      }
      event.preventDefault();
      focusable[0]?.focus();
    }
    window.addEventListener("keydown", trap);
    return () => window.removeEventListener("keydown", trap);
  }, [open]);

  async function run(item: CommandItem) {
    rememberCommand(item.id);
    if (item.kind === "help") {
      setReceipt(COMMAND_INDEX_HONESTY);
      return;
    }
    if (item.execution === "copy-location" || item.execution === "copy-ref") {
      const value =
        item.execution === "copy-ref" ? (item.copyValue ?? "") : window.location.hash || "#/";
      try {
        await navigator.clipboard.writeText(value);
        setReceipt(`Copied ${value}`);
      } catch {
        setMessage("Copy is unavailable in this environment.");
      }
      return;
    }
    if (item.execution === "acknowledge" && item.alertId) {
      setMessage(undefined);
      const result = await readJson("/management/alerts/acknowledge", "management", {
        method: "POST",
        headers: { "content-type": "application/json" },
        body: JSON.stringify({ alert_id: item.alertId }),
      });
      if (result.ok) {
        setReceipt(`Alert ${item.alertId} acknowledged.`);
        recordSessionMutation(appProjections, {
          id: `alert.acknowledge:${item.alertId}`,
          action: "alert.acknowledge",
          objectRef: item.alertId,
          atMs: Date.now(),
          detail: "budget alert acknowledged from the command palette",
        });
        await fetchProjection(
          appProjections,
          "home:alerts",
          "/management/alerts",
          "management",
          projectProviderAlerts,
        );
      } else {
        setMessage(`HTTP ${result.status} ${String(asRecord(result.body).code ?? "")}`);
      }
      return;
    }
    if (item.href) {
      navigate(item.href);
      close();
    }
  }

  if (!open) {
    return null;
  }

  const selected = items[cursor];

  return (
    <div className="cp-palette-scrim" onMouseDown={close}>
      <div
        ref={dialogRef}
        className="cp-palette"
        role="dialog"
        aria-modal="true"
        aria-label="Command palette"
        onMouseDown={(event) => event.stopPropagation()}
      >
        <HonestyNote>{COMMAND_INDEX_HONESTY}</HonestyNote>
        <p className="cp-quiet">{COMMAND_NO_CLASS_C}</p>
        <input
          ref={inputRef}
          className="cp-palette-input"
          type="text"
          name="command-palette-query"
          autoComplete="off"
          autoCapitalize="off"
          spellCheck={false}
          role="combobox"
          aria-expanded="true"
          aria-controls="cp-palette-list"
          aria-activedescendant={selected ? `cp-palette-${selected.id}` : undefined}
          aria-autocomplete="list"
          placeholder="Search objects, actions, destinations…"
          value={query}
          onChange={(event) => setQuery(event.target.value)}
          onKeyDown={(event) => {
            if (event.key === "ArrowDown") {
              event.preventDefault();
              setCursor((index) => Math.min(items.length - 1, index + 1));
            } else if (event.key === "ArrowUp") {
              event.preventDefault();
              setCursor((index) => Math.max(0, index - 1));
            } else if (event.key === "Enter" && selected) {
              event.preventDefault();
              void run(selected);
            }
          }}
        />
        <ul id="cp-palette-list" className="cp-palette-list" role="listbox">
          {items.length === 0 ? (
            <li className="cp-quiet" role="presentation">
              {COMMAND_NO_RESULTS}
            </li>
          ) : (
            groups.flatMap((group) => [
              <li key={`group:${group.kind}`} className="cp-palette-group" role="presentation">
                {group.label}
              </li>,
              ...group.items.map((item) => {
                const index = items.findIndex((entry) => entry.id === item.id);
                return (
                  <li
                    key={item.id}
                    id={`cp-palette-${item.id}`}
                    role="option"
                    aria-selected={index === cursor}
                    className={index === cursor ? "cp-palette-row is-active" : "cp-palette-row"}
                    onMouseEnter={() => setCursor(index)}
                    onClick={() => void run(item)}
                  >
                    <span className="cp-palette-kind">{item.kind}</span>
                    <span className="cp-palette-label">{item.label}</span>
                    {item.detail ? <span className="cp-quiet">{item.detail}</span> : null}
                  </li>
                );
              }),
            ])
          )}
        </ul>
        {receipt ? <ReceiptLine>{receipt}</ReceiptLine> : null}
        {message ? (
          <p role="alert" className="cp-reason">
            {message}
          </p>
        ) : null}
      </div>
    </div>
  );
}
