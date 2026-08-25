import { CATEGORY_META, type StateCategory } from "./stateMap";

/**
 * Category dot — shape + color, never the only signal (a label always
 * accompanies it via StateChip or row text). Shape is encoded in CSS.
 */
export function StateDot({ category }: { category: StateCategory }) {
  const meta = CATEGORY_META[category];
  return (
    <span
      className={`cp-dot cp-dot--${category}`}
      aria-hidden="true"
      data-shape={meta.shape}
    />
  );
}
