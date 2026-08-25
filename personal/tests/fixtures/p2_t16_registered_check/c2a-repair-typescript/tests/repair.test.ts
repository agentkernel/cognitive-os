import { add } from "../src/repair";

if (add(2, 3) !== 5) {
  throw new Error("repair failed");
}
