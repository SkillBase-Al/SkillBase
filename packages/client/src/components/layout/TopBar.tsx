import * as React from 'react';
import { Search, Settings } from 'lucide-react';
import { Input } from '../ui/input';

interface TopBarProps {
  searchQuery: string;
  onSearchChange: (query: string) => void;
  searchPlaceholder?: string;
  onSettingsClick?: () => void;
}

function TopBar({
  searchQuery,
  onSearchChange,
  searchPlaceholder = 'Search skills...',
  onSettingsClick,
}: TopBarProps) {
  return (
    <header className="flex items-center gap-4 h-14 border-b border-slate-200 dark:border-slate-700 bg-white dark:bg-slate-900 px-4">
      <div className="flex-1 max-w-md relative">
        <Search className="absolute left-3 top-1/2 -translate-y-1/2 h-4 w-4 text-slate-400" />
        <Input
          value={searchQuery}
          onChange={(e) => onSearchChange(e.target.value)}
          placeholder={searchPlaceholder}
          className="pl-9 h-9 text-sm"
        />
      </div>
      {onSettingsClick && (
        <button
          onClick={onSettingsClick}
          className="p-2 rounded-md text-slate-500 dark:text-slate-400 hover:bg-slate-100 dark:hover:bg-slate-800 transition-colors"
          title="Settings"
        >
          <Settings className="h-5 w-5" />
        </button>
      )}
    </header>
  );
}

export { TopBar };
