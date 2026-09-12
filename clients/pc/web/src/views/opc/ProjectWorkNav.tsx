import { NavLink } from "react-router-dom";

export const PROJECT_WORK_NAV = [
  { to: "", label: "详情" },
  { to: "/members", label: "成员" },
  { to: "/runs", label: "运行" },
  { to: "/outputs", label: "产出" },
] as const;

export function ownerProjectStatus(state: string): string {
  switch (state) {
    case "creating":
      return "未上线";
    case "active":
      return "已上线";
    case "attention":
      return "需关注";
    case "paused":
      return "已暂停";
    default:
      return state;
  }
}

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
