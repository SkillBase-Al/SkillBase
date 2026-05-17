import * as React from 'react';
import { useNavigate } from 'react-router-dom';
import {
  Compass,
  WifiOff,
  RefreshCw,
  ChevronLeft,
  ChevronRight,
} from 'lucide-react';
import { TopBar } from '../components/layout/TopBar';
import { SkillGrid } from '../components/skill/SkillGrid';
import { SkillCard } from '../components/skill/SkillCard';
import { SkillDetail } from '../components/skill/SkillDetail';
import { Tabs, TabsList, TabsTrigger } from '../components/ui/tabs';
import { Badge } from '../components/ui/badge';
import { Button } from '../components/ui/button';
import { useToast } from '../components/ui/toast';
import { useMarketStore } from '../stores/marketStore';
import { useInstalledStore } from '../stores/installedStore';
import { getCategories } from '../services/tauri';
import type { MarketSkill } from '../types/skill';

function DiscoverPage() {
  const navigate = useNavigate();
  const { skills, categories, isLoading, error, search, currentPage, totalPages, total, perPage } = useMarketStore();
  const { install, items: installedItems } = useInstalledStore();
  const { addToast } = useToast();
  const incrementDownloads = useMarketStore((s) => s.incrementDownloads);

  const installedNames = React.useMemo(() => new Set(installedItems.map((s) => s.name)), [installedItems]);

  const [searchQuery, setSearchQuery] = React.useState('');
  const [activeCategory, setActiveCategory] = React.useState('all');
  const [selectedSkill, setSelectedSkill] = React.useState<MarketSkill | null>(null);
  const [detailOpen, setDetailOpen] = React.useState(false);
  const [cachedCategories, setCachedCategories] = React.useState<string[]>([]);
  const [isOffline, setIsOffline] = React.useState(false);

  // Load categories and fetch initial skills on mount
  React.useEffect(() => {
    getCategories()
      .then(setCachedCategories)
      .catch(() => setIsOffline(true));
    search('', undefined, 1);
  }, [search]);

  // Search with 300ms debounce on query/category change (resets to page 1)
  // Page changes via pagination buttons call search directly and skip debounce
  const previousQueryRef = React.useRef('');
  const previousCategoryRef = React.useRef('all');
  React.useEffect(() => {
    const q = searchQuery.trim();
    const cat = activeCategory;
    if (q === previousQueryRef.current && cat === previousCategoryRef.current) return;
    previousQueryRef.current = q;
    previousCategoryRef.current = cat;

    const timer = setTimeout(() => {
      search(q, cat === 'all' ? undefined : cat, 1);
    }, 300);
    return () => clearTimeout(timer);
  }, [searchQuery, activeCategory, search]);

  const handleSkillClick = (skill: MarketSkill) => {
    setSelectedSkill(skill);
    setDetailOpen(true);
  };

  const handleInstall = async () => {
    if (!selectedSkill) return;
    try {
      await install(selectedSkill.packageId);
      incrementDownloads(selectedSkill.packageId);
      addToast(`"${selectedSkill.name}" installed successfully`, 'success');
      setDetailOpen(false);
    } catch {
      // Error handled by store
    }
  };

  return (
    <div className="flex flex-col h-full">
      <TopBar
        searchQuery={searchQuery}
        onSearchChange={setSearchQuery}
        searchPlaceholder="Discover skills..."
        onSettingsClick={() => navigate('/settings')}
      />

      {/* Offline banner */}
      {isOffline && (
        <div className="flex items-center gap-2 px-4 py-2 bg-yellow-50 dark:bg-yellow-950 border-b border-yellow-200 dark:border-yellow-800">
          <WifiOff className="h-4 w-4 text-yellow-600" />
          <span className="text-xs text-yellow-700 dark:text-yellow-300">
            You appear to be offline. Showing cached data.
          </span>
        </div>
      )}

      <div className="flex-1 overflow-auto p-4 space-y-4">
        {/* Category Tabs */}
        <div className="flex items-center gap-2">
          <Compass className="h-5 w-5 text-blue-600" />
          <span className="text-sm font-semibold text-slate-700 dark:text-slate-300">
            Discover
          </span>
          {!isLoading && total > 0 && (
            <Badge variant="outline" className="text-xs">
              {total} results
            </Badge>
          )}
        </div>

        <Tabs defaultValue="all" value={activeCategory} onValueChange={setActiveCategory}>
          <TabsList className="flex-wrap h-auto gap-1 p-1">
            <TabsTrigger value="all">All</TabsTrigger>
            {[...new Set([...cachedCategories, ...categories])].map((cat) => (
              <TabsTrigger key={cat} value={cat}>
                {cat.charAt(0).toUpperCase() + cat.slice(1)}
              </TabsTrigger>
            ))}
          </TabsList>
        </Tabs>

        {/* Error with retry */}
        {error && !isLoading && (
          <div className="flex items-center gap-3 p-4 rounded-lg border border-red-200 dark:border-red-800 bg-red-50 dark:bg-red-950">
            <p className="text-sm text-red-600 dark:text-red-400 flex-1">
              {error}
            </p>
            <button
              onClick={() => {
                const q = searchQuery.trim();
                search(q, activeCategory === 'all' ? undefined : activeCategory, 1);
              }}
              className="flex items-center gap-1 text-sm text-red-600 hover:text-red-700"
            >
              <RefreshCw className="h-4 w-4" />
              Retry
            </button>
          </div>
        )}

        {/* Skill Grid */}
        <SkillGrid
          skills={skills}
          isLoading={isLoading}
          error={null}
          isEmpty={skills.length === 0 && !isLoading}
          renderCard={(skill) => {
            const marketSkill = skill as MarketSkill;
            const isInstalled = installedNames.has(marketSkill.name);
            return (
              <SkillCard
                skill={skill}
                onClick={() => handleSkillClick(marketSkill)}
                onInstall={async () => {
                  if (!isInstalled) {
                    try {
                      await install(marketSkill.packageId);
                      incrementDownloads(marketSkill.packageId);
                      addToast(`"${marketSkill.name}" installed successfully`, 'success');
                    } catch {
                      // Error handled by store
                    }
                  }
                }}
                showInstallButton={!isInstalled}
              />
            );
          }}
        />

        {/* Pagination */}
        {totalPages > 1 && !isLoading && (
          <div className="flex items-center justify-center gap-1.5 py-3">
            <Button
              variant="outline"
              size="sm"
              disabled={currentPage <= 1}
              onClick={() => useMarketStore.getState().search(
                searchQuery.trim(),
                activeCategory === 'all' ? undefined : activeCategory,
                currentPage - 1,
              )}
            >
              <ChevronLeft className="h-4 w-4" />
            </Button>
            {Array.from({ length: totalPages }, (_, i) => i + 1).map((p) => {
              // Show first, last, and pages near current
              const show = p === 1 || p === totalPages || Math.abs(p - currentPage) <= 1;
              const prev = p - 1;
              if (!show && prev >= 1 && (prev === 1 || prev === totalPages || Math.abs(prev - currentPage) <= 1)) {
                // Show ellipsis before this page
                return <span key={p} className="text-xs text-slate-400 px-1">...</span>;
              }
              if (!show) return null;
              return (
                <Button
                  key={p}
                  variant={p === currentPage ? 'default' : 'outline'}
                  size="sm"
                  className="min-w-[32px]"
                  onClick={() => {
                    if (p !== currentPage) {
                      useMarketStore.getState().search(
                        searchQuery.trim(),
                        activeCategory === 'all' ? undefined : activeCategory,
                        p,
                      );
                    }
                  }}
                >
                  {p}
                </Button>
              );
            })}
            <Button
              variant="outline"
              size="sm"
              disabled={currentPage >= totalPages}
              onClick={() => useMarketStore.getState().search(
                searchQuery.trim(),
                activeCategory === 'all' ? undefined : activeCategory,
                currentPage + 1,
              )}
            >
              <ChevronRight className="h-4 w-4" />
            </Button>
          </div>
        )}
      </div>

      {/* Detail Panel */}
      <SkillDetail
        skill={selectedSkill}
        open={detailOpen}
        onOpenChange={setDetailOpen}
        onInstall={handleInstall}
        isInstalled={selectedSkill ? installedNames.has(selectedSkill.name) : false}
      />
    </div>
  );
}

export { DiscoverPage };
