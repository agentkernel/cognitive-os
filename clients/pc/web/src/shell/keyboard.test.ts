import { afterEach, describe, expect, it } from "vitest";
import {
  SPACE_CHORDS,
  findBackLink,
  isTypingTarget,
  openSelectedMaster,
  registerInspectorClear,
  stepDetailSection,
  stepMaster,
  tryClearInspector,
  unwindDetail,
} from "./keyboard";

function mountMain(html: string): HTMLElement {
  const main = document.createElement("main");
  main.innerHTML = html;
  document.body.appendChild(main);
  return main;
}

afterEach(() => {
  for (const main of document.querySelectorAll("main")) {
    main.remove();
  }
});

describe("shell keyboard helpers (W12)", () => {
  it("maps g-chords onto the seven spaces", () => {
    expect(SPACE_CHORDS).toEqual({
      h: "/",
      w: "/work",
      a: "/agents",
      p: "/providers",
      r: "/resources",
      c: "/activity",
      s: "/system",
    });
  });

  it("treats inputs as typing targets and leaves buttons alone", () => {
    const input = document.createElement("input");
    const button = document.createElement("button");
    expect(isTypingTarget(input)).toBe(true);
    expect(isTypingTarget(button)).toBe(false);
  });

  it("j/k inspect the neighbouring master row", () => {
    const main = mountMain(`
      <table>
        <tbody>
          <tr data-row-key="a" aria-selected="true">
            <td>A</td>
            <td><button type="button">Inspect</button><a href="#/work/a">Open</a></td>
          </tr>
          <tr data-row-key="b">
            <td>B</td>
            <td><button type="button">Inspect</button><a href="#/work/b">Open</a></td>
          </tr>
        </tbody>
      </table>
    `);
    const clicks: string[] = [];
    for (const button of main.querySelectorAll("button")) {
      button.addEventListener("click", () => {
        clicks.push((button.closest("tr") as HTMLElement).dataset.rowKey ?? "");
      });
    }
    expect(stepMaster(1, document)).toBe(true);
    expect(clicks).toEqual(["b"]);
    main.remove();
  });

  it("Enter inspects first, then opens the selected row", () => {
    const main = mountMain(`
      <table>
        <tbody>
          <tr data-row-key="a">
            <td>A</td>
            <td><button type="button">Inspect</button><a href="#/work/a">Open</a></td>
          </tr>
        </tbody>
      </table>
    `);
    const events: string[] = [];
    main.querySelector("button")?.addEventListener("click", () => events.push("inspect"));
    main.querySelector("a")?.addEventListener("click", (event) => {
      event.preventDefault();
      events.push("open");
    });
    expect(openSelectedMaster(document)).toBe(true);
    expect(events).toEqual(["inspect"]);
    main.querySelector("tr")?.setAttribute("aria-selected", "true");
    expect(openSelectedMaster(document)).toBe(true);
    expect(events).toEqual(["inspect", "open"]);
    main.remove();
  });

  it("bracket keys walk the detail section navigator", () => {
    const main = mountMain(`
      <nav>
        <button class="cp-sectionnav-link" aria-current="true">Overview</button>
        <button class="cp-sectionnav-link">Run</button>
        <button class="cp-sectionnav-link">Effects</button>
        <button class="cp-sectionnav-link">Evidence</button>
      </nav>
    `);
    const labels: string[] = [];
    for (const button of main.querySelectorAll("button")) {
      button.addEventListener("click", () => {
        for (const other of main.querySelectorAll("button")) {
          other.removeAttribute("aria-current");
        }
        button.setAttribute("aria-current", "true");
        labels.push((button.textContent ?? "").trim());
      });
    }
    expect(stepDetailSection(1, document)).toBe(true);
    expect(stepDetailSection(1, document)).toBe(true);
    expect(stepDetailSection(1, document)).toBe(true);
    expect(labels).toEqual(["Run", "Effects", "Evidence"]);
    main.remove();
  });

  it("Escape clicks a Back-to link, then clears a registered inspector", () => {
    const main = mountMain(`<a href="#/work">Back to Work</a>`);
    let back = 0;
    main.querySelector("a")?.addEventListener("click", (event) => {
      event.preventDefault();
      back += 1;
    });
    expect(unwindDetail(document)).toBe(true);
    expect(back).toBe(1);
    main.remove();

    let cleared = 0;
    const stop = registerInspectorClear(() => {
      cleared += 1;
      return true;
    });
    expect(tryClearInspector()).toBe(true);
    expect(cleared).toBe(1);
    stop();
    expect(tryClearInspector()).toBe(false);
  });

  it("finds Back-to links by their visible label", () => {
    const main = mountMain(`<a href="#/agents">Back to Agents</a>`);
    expect(findBackLink(document)?.getAttribute("href")).toBe("#/agents");
    main.remove();
  });
});
