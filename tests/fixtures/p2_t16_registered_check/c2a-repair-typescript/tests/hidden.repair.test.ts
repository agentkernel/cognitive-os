import { add } from "../src/repair";

if (add(4, 1) !== 5) {
  throw new Error("hidden repair failed");
}
