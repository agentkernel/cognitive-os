import { useEffect, useRef } from "react";
import { useNavigate } from "react-router-dom";
import {
  G_CHORD_MS,
  SPACE_CHORDS,
  isEnterReserved,
  isTypingTarget,
  openSelectedMaster,
  stepDetailSection,
  stepMaster,
  tryClearInspector,
  unwindDetail,
} from "./keyboard";

/**
 * Design-12 keyboard layer. Bound on the window so it works from any space.
 * Typing in a field, and the open palette, keep their own keys.
 */
export function useShellKeyboard(options: {
  paletteOpen: boolean;
  onOpenPalette: () => void;
}): void {
  const navigate = useNavigate();
  const armed = useRef(0);
  const { paletteOpen, onOpenPalette } = options;

  useEffect(() => {
    function onKey(event: KeyboardEvent) {
      if (event.metaKey || event.ctrlKey || event.altKey) {
        return;
      }
      if (isTypingTarget(event.target)) {
        return;
      }
      if (paletteOpen) {
        return;
      }

      const key = event.key;
      const now = Date.now();

      if (armed.current > 0 && now - armed.current <= G_CHORD_MS) {
        const dest = SPACE_CHORDS[key.toLowerCase()];
        armed.current = 0;
        if (dest) {
          event.preventDefault();
          navigate(dest);
        }
        return;
      }
      armed.current = 0;

      if (key.toLowerCase() === "g" && !event.repeat) {
        armed.current = now;
        event.preventDefault();
        return;
      }
      if (key === "/") {
        event.preventDefault();
        onOpenPalette();
        return;
      }
      if (key === "Escape") {
        if (unwindDetail() || tryClearInspector()) {
          event.preventDefault();
        }
        return;
      }
      if (key === "j") {
        if (stepMaster(1)) {
          event.preventDefault();
        }
        return;
      }
      if (key === "k") {
        if (stepMaster(-1)) {
          event.preventDefault();
        }
        return;
      }
      if (key === "Enter") {
        if (isEnterReserved(event.target)) {
          return;
        }
        if (openSelectedMaster()) {
          event.preventDefault();
        }
        return;
      }
      if (key === "[" || key === "]") {
        const delta = key === "]" ? 1 : -1;
        if (stepDetailSection(delta) || stepMaster(delta)) {
          event.preventDefault();
        }
      }
    }

    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [navigate, onOpenPalette, paletteOpen]);
}
