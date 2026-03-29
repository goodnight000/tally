import { useLocation, useNavigate } from "react-router-dom";
import { ToolIcon } from "../shared/ToolIcon";
import { TallyLogo } from "../shared/TallyLogo";
import { useSources } from "../../contexts/SourceContext";

type NavItem = {
  path: string;
  label: string;
  icon?: string;
  tool?: string;
};

export function Sidebar() {
  const location = useLocation();
  const navigate = useNavigate();
  const { sources } = useSources();

  const enabledSources = sources.filter((s) => s.enabled);

  const navItems: NavItem[] = [
    { path: "/", label: "Home", icon: "⌂" },
    ...enabledSources.map((s) => ({
      path: `/tool/${s.id}`,
      label: s.display_name,
      tool: s.id,
    })),
    { path: "/sessions", label: "Sessions", icon: "☰" },
    { path: "/settings", label: "Settings", icon: "⚙" },
  ];

  return (
    <aside className="w-(--spacing-sidebar-collapsed) md:w-(--spacing-sidebar) bg-white border-r border-border flex flex-col shrink-0 transition-all duration-300">
      <div className="p-2 md:p-6 pb-2 md:pb-4">
        <div className="flex items-center gap-3 justify-center md:justify-start">
          <TallyLogo size={28} className="shrink-0" />
          <h1 className="font-serif text-xl font-semibold text-text-primary tracking-tight hidden md:block">
            Tally
          </h1>
        </div>
      </div>
      <nav className="flex-1 px-1 md:px-3">
        {navItems.map((item) => {
          const isActive = location.pathname === item.path;
          return (
            <button
              key={item.path}
              onClick={() => navigate(item.path)}
              title={item.label}
              className={`w-full text-left px-2 md:px-3 py-2 rounded-lg mb-0.5 text-sm transition-all duration-300 flex items-center justify-center md:justify-start gap-2.5 ${
                isActive
                  ? "bg-cream text-text-primary font-medium"
                  : "text-text-secondary hover:bg-cream/50 hover:text-text-primary"
              }`}
            >
              {item.tool ? (
                <ToolIcon tool={item.tool} size={16} />
              ) : (
                <span className="text-base leading-none w-4 text-center">
                  {item.icon}
                </span>
              )}
              <span className="hidden md:inline">{item.label}</span>
            </button>
          );
        })}
      </nav>
    </aside>
  );
}
