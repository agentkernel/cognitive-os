import { NavLink } from "react-router-dom";

export const PROJECT_WORK_NAV = [
  { to: "", label: "Detail" },
  { to: "/members", label: "Members" },
  { to: "/runs", label: "Runs" },
  { to: "/outputs", label: "Outputs" },
] as const;

/**
 * Project L2: 详情 / 成员 / 运行 / 产出. Not Team, not Inbox, not Work.
 */
export function ProjectWorkNav({ projectId }: { projectId: string }) {
  const base = `/projects/${encodeURIComponent(projectId)}`;
  return (
    <nav className="cp-subnav" aria-label="Project sections" data-nav="project-work">
      {PROJECT_WORK_NAV.map((item) => (
        <NavLink
          key={item.label}
          to={`${base}${item.to}`}
          end={item.to === ""}
          className="cp-button"
        >
          {item.label}
        </NavLink>
      ))}
    </nav>
  );
}
