import { DETAIL_SECTIONS, type DetailSectionId } from "../../../data/projections/workDetail";

/**
 * Section navigator — docs/design/15 §2. The six sections are one continuous
 * page: this moves the viewport, it does not hide anything. Tabs and
 * accordions were rejected because they let a reader believe they have seen a
 * task when they have only seen one panel of it.
 */
export function SectionNavigator({
  active,
  onSelect,
}: {
  active: DetailSectionId;
  onSelect: (section: DetailSectionId) => void;
}) {
  return (
    <nav className="cp-sectionnav" aria-label="Task detail sections">
      <ul className="cp-sectionnav-list">
        {DETAIL_SECTIONS.map((section) => (
          <li key={section.id}>
            <button
              type="button"
              className="cp-sectionnav-link"
              aria-current={active === section.id ? "true" : undefined}
              onClick={() => onSelect(section.id)}
            >
              {section.title}
            </button>
          </li>
        ))}
      </ul>
      <p className="cp-quiet">
        Every section below is always rendered. This only moves you to one of them.
      </p>
    </nav>
  );
}
