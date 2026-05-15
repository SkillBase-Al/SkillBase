import * as React from 'react';
import { NavLink } from 'react-router-dom';
import {
  Compass,
  Package,
  GitCompareArrows,
  Settings,
  ChevronLeft,
  ChevronRight,
  Brain,
} from 'lucide-react';
import { cn } from '../../lib/utils';

interface SidebarProps {
  collapsed: boolean;
  onToggle: () => void;
}

const navItems = [
  { to: '/discover', icon: Compass, label: 'Discover' },
  { to: '/installed', icon: Package, label: 'Installed' },
  { to: '/dedup', icon: GitCompareArrows, label: 'Dedup' },
  { to: '/settings', icon: Settings, label: 'Settings' },
];

function Sidebar({ collapsed, onToggle }: SidebarProps) {
  return (
    <aside
      className={cn(
        'flex flex-col border-r border-slate-200 dark:border-slate-700 bg-white dark:bg-slate-900 transition-all duration-200',
        collapsed ? 'w-16' : 'w-56',
      )}
    >
      {/* Logo */}
      <div
        className={cn(
          'flex items-center h-14 border-b border-slate-200 dark:border-slate-700 px-4',
          collapsed ? 'justify-center' : 'gap-3',
        )}
      >
        <div className="flex items-center justify-center h-8 w-8 rounded-lg bg-blue-600 text-white">
          <Brain className="h-4 w-4" />
        </div>
        {!collapsed && (
          <span className="font-semibold text-slate-800 dark:text-slate-200 text-sm">
            SkillBase
          </span>
        )}
      </div>

      {/* Navigation */}
      <nav className="flex-1 p-2 space-y-1">
        {navItems.map((item) => (
          <NavLink
            key={item.to}
            to={item.to}
            className={({ isActive }) =>
              cn(
                'flex items-center gap-3 rounded-md px-3 py-2 text-sm font-medium transition-colors',
                isActive
                  ? 'bg-blue-50 dark:bg-blue-950 text-blue-700 dark:text-blue-300'
                  : 'text-slate-600 dark:text-slate-400 hover:bg-slate-100 dark:hover:bg-slate-800 hover:text-slate-800 dark:hover:text-slate-200',
                collapsed && 'justify-center px-2',
              )
            }
          >
            <item.icon className="h-5 w-5 flex-shrink-0" />
            {!collapsed && <span>{item.label}</span>}
          </NavLink>
        ))}
      </nav>

      {/* Collapse toggle */}
      <div className="border-t border-slate-200 dark:border-slate-700 p-2">
        <button
          onClick={onToggle}
          className={cn(
            'flex items-center gap-3 w-full rounded-md px-3 py-2 text-sm text-slate-500 dark:text-slate-400 hover:bg-slate-100 dark:hover:bg-slate-800 transition-colors',
            collapsed && 'justify-center px-2',
          )}
          title={collapsed ? 'Expand sidebar' : 'Collapse sidebar'}
        >
          {collapsed ? (
            <ChevronRight className="h-5 w-5" />
          ) : (
            <>
              <ChevronLeft className="h-5 w-5" />
              <span>Collapse</span>
            </>
          )}
        </button>
      </div>
    </aside>
  );
}

export { Sidebar };
