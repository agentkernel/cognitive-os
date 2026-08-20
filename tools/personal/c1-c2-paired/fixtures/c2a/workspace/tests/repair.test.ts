import { add } from "../src/repair.ts";

if (add(2, 3) !== 5) {
  throw new Error("hidden: add must return 5");
}
