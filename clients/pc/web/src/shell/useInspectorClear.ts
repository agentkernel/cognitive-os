import { useEffect } from "react";
import { registerInspectorClear } from "./keyboard";

/** Lets a master page clear its inspector on Escape (design-12 unwind). */
export function useInspectorClear(selected: string | undefined, clear: () => void): void {
  useEffect(() => {
    return registerInspectorClear(() => {
      if (!selected) {
        return false;
      }
      clear();
      return true;
    });
  }, [selected, clear]);
}
