import { useCallback, useEffect, useMemo } from "react";
import { PageHeader } from "../../components/PageHeader";
import { fetchProjection } from "../../data/fetchProjection";
import {
  composeFamilyRows,
  HUB_FAMILIES,
  projectResourceList,
  resourceListKey,
  resourceListPath,
  type ResourceListView,
} from "../../data/projections/resources";
import { appProjections } from "../../data/store";
import { useProjection } from "../../data/useProjection";
import { HonestyNote } from "../../state/HonestyNote";
import { FamilyIndex } from "./FamilyIndex";

/**
 * Resources hub — docs/design/18 §1. Four family rows over real list reads.
 * Mutations and family-page depth are not this slice.
 */
export function ResourcesPage() {
  const refresh = useCallback(async () => {
    await Promise.all(
      HUB_FAMILIES.map((family) =>
        fetchProjection(
          appProjections,
          resourceListKey(family.id),
          resourceListPath(family.id),
          "management",
          projectResourceList,
        ),
      ),
    );
  }, []);

  useEffect(() => {
    void refresh();
  }, [refresh]);

  const memory = useProjection<ResourceListView>(resourceListKey("memory"));
  const skill = useProjection<ResourceListView>(resourceListKey("skill"));
  const tool = useProjection<ResourceListView>(resourceListKey("tool"));
  const context = useProjection<ResourceListView>(resourceListKey("context"));

  const rows = useMemo(
    () => composeFamilyRows({ memory, skill, tool, context }),
    [memory, skill, tool, context],
  );

  return (
    <section>
      <PageHeader
        title="Resources"
        lede="Four families. Counts come from the Resource Manager list envelope. Context is per-task, not a browser."
      />
      <p className="cp-next">
        <button type="button" className="cp-button" onClick={() => void refresh()}>
          Refresh
        </button>{" "}
        <span className="cp-quiet">
          This space refreshes only when you ask. Nothing on this page polls the daemon.
        </span>
      </p>

      <HonestyNote>
        This hub is a family index, not a card wall. Memory, Skills and Tools counts come from{" "}
        <code>GET /management/resource/v1/list?family=</code> (limit 64). Memory list is
        non-tombstoned admitted rows; skill list is bindings, not packages. Context&apos;s list is
        projection-only — open a task from Work. Remember, import, enable, forget and quarantine
        are not on this page.
      </HonestyNote>

      <FamilyIndex rows={rows} />
    </section>
  );
}
