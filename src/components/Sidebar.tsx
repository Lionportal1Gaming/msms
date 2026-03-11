interface SidebarProps {
  activeView: string;
  onSelect: (view: string) => void;
}

const items = [
  ["dashboard", "Dashboard"],
  ["provision", "Provision"],
  ["config", "Config"],
  ["runtimes", "Runtimes"],
  ["backups", "Backups"],
  ["console", "Console"],
  ["settings", "Settings"]
];

export function Sidebar({ activeView, onSelect }: SidebarProps) {
  return (
    <aside className="sidebar">
      <div>
        <p className="eyebrow">MSMS</p>
        <h1>Minecraft Server Management System</h1>
      </div>
      <nav className="sidebar-nav">
        {items.map(([value, label]) => (
          <button
            key={value}
            className={activeView === value ? "nav-button active" : "nav-button"}
            onClick={() => onSelect(value)}
            type="button"
          >
            {label}
          </button>
        ))}
      </nav>
    </aside>
  );
}
