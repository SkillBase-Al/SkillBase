import * as React from 'react';
import { useNavigate } from 'react-router-dom';
import {
  Compass,
  WifiOff,
  RefreshCw,
} from 'lucide-react';
import { TopBar } from '../components/layout/TopBar';
import { SkillGrid } from '../components/skill/SkillGrid';
import { SkillCard } from '../components/skill/SkillCard';
import { SkillDetail } from '../components/skill/SkillDetail';
import { Tabs, TabsList, TabsTrigger } from '../components/ui/tabs';
import { Badge } from '../components/ui/badge';
import { useMarketStore } from '../stores/marketStore';
import { useInstalledStore } from '../stores/installedStore';
import { getCategories } from '../services/tauri';
import type { MarketSkill } from '../types/skill';

function DiscoverPage() {
  const navigate = useNavigate();
  const { skills, categories, isLoading, error, search } = useMarketStore();
  const { install } = useInstalledStore();

  const [searchQuery, setSearchQuery] = React.useState('');
  const [activeCategory, setActiveCategory] = React.useState('all');
  const [selectedSkill, setSelectedSkill] = React.useState<MarketSkill | null>(null);
  const [detailOpen, setDetailOpen] = React.useState(false);
  const [cachedCategories, setCachedCategories] = React.useState<string[]>([]);
  const [isOffline, setIsOffline] = React.useState(false);

  // Load categories on mount
  React.useEffect(() => {
    getCategories()
      .then(setCachedCategories)
      .catch(() => setIsOffline(true));
  }, []);

  // Search with 300ms debounce
  const previousQueryRef = React.useRef('');
  const previousCategoryRef = React.useRef('all');
  React.useEffect(() => {
    const q = searchQuery.trim();
    const cat = activeCategory;
    if (q === previousQueryRef.current && cat === previousCategoryRef.current) return;
    previousQueryRef.current = q;
    previousCategoryRef.current = cat;

    const timer = setTimeout(() => {
      search(q, cat === 'all' ? undefined : cat);
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
          {!isLoading && skills.length > 0 && (
            <Badge variant="outline" className="text-xs">
              {skills.length} results
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
                search(q, activeCategory === 'all' ? undefined : activeCategory);
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
          renderCard={(skill) => (
            <SkillCard
              skill={skill}
              onClick={() => handleSkillClick(skill as MarketSkill)}
              onInstall={() => {
                const marketSkill = skill as MarketSkill;
                if (!marketSkill.isInstalled) {
                  install(marketSkill.packageId).catch(() => {});
                }
              }}
              showInstallButton={!(skill as MarketSkill).isInstalled}
            />
          )}
        />
      </div>

      {/* Detail Panel */}
      <SkillDetail
        skill={selectedSkill}
        open={detailOpen}
        onOpenChange={setDetailOpen}
        onInstall={handleInstall}
        isInstalled={selectedSkill?.isInstalled ?? false}
      />
    </div>
  );
}

export { DiscoverPage };
