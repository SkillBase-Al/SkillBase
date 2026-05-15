import * as React from 'react';
import { BrowserRouter, Routes, Route, Navigate } from 'react-router-dom';
import { AppLayout } from './components/layout/AppLayout';
import { FirstRunWizard } from './components/layout/FirstRunWizard';
import { ToastProvider } from './components/ui/toast';
import { DiscoverPage } from './pages/DiscoverPage';
import { InstalledPage } from './pages/InstalledPage';
import { DedupPage } from './pages/DedupPage';
import { SettingsPage } from './pages/SettingsPage';
import { ConflictResolver } from './components/skill/ConflictResolver';
import { checkFirstRun, getSkillConflicts } from './services/tauri';

function App() {
  const [isFirstRun, setIsFirstRun] = React.useState<boolean | null>(null);
  const [conflictOpen, setConflictOpen] = React.useState(false);

  React.useEffect(() => {
    checkFirstRun()
      .then((firstRun) => setIsFirstRun(firstRun))
      .catch(() => setIsFirstRun(false));
  }, []);

  // Check for skill conflicts on startup (after first run check)
  React.useEffect(() => {
    if (isFirstRun === false) {
      getSkillConflicts()
        .then((conflicts) => {
          if (conflicts.length > 0) {
            setConflictOpen(true);
          }
        })
        .catch(() => {});
    }
  }, [isFirstRun]);

  // Loading while checking first run
  if (isFirstRun === null) {
    return (
      <div className="flex items-center justify-center h-screen bg-slate-50 dark:bg-slate-950">
        <div className="flex flex-col items-center gap-3">
          <div className="h-8 w-8 rounded-full border-2 border-blue-600 border-t-transparent animate-spin" />
          <p className="text-sm text-slate-500">Loading...</p>
        </div>
      </div>
    );
  }

  return (
    <ToastProvider>
      <BrowserRouter>
        {isFirstRun && (
          <FirstRunWizard onComplete={() => setIsFirstRun(false)} />
        )}
        <Routes>
          <Route element={<AppLayout />}>
            <Route path="/discover" element={<DiscoverPage />} />
            <Route path="/installed" element={<InstalledPage />} />
            <Route path="/dedup" element={<DedupPage />} />
            <Route path="/settings" element={<SettingsPage />} />
            <Route path="/" element={<Navigate to="/discover" replace />} />
            <Route path="*" element={<Navigate to="/discover" replace />} />
          </Route>
        </Routes>

        <ConflictResolver
          open={conflictOpen}
          onOpenChange={setConflictOpen}
          onAllResolved={() => setConflictOpen(false)}
        />
      </BrowserRouter>
    </ToastProvider>
  );
}

export default App;
