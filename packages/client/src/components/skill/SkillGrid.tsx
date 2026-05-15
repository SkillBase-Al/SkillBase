import * as React from 'react';
import { SkillCard } from './SkillCard';
import { LoadingSkeleton } from '../shared/LoadingSkeleton';
import { EmptyState } from '../shared/EmptyState';
import { ErrorState } from '../shared/ErrorState';
import { PackageSearch } from 'lucide-react';
import type { InstalledSkill, MarketSkill } from '../../types/skill';

type Skill = InstalledSkill | MarketSkill;

interface SkillGridProps {
  skills: Skill[];
  isLoading: boolean;
  error: string | null;
  isEmpty: boolean;
  onRetry?: () => void;
  renderCard?: (skill: Skill) => React.ReactNode;
}

function SkillGrid({
  skills,
  isLoading,
  error,
  isEmpty,
  onRetry,
  renderCard,
}: SkillGridProps) {
  if (isLoading) {
    return <LoadingSkeleton variant="card" count={6} />;
  }

  if (error) {
    return <ErrorState message={error} onRetry={onRetry} />;
  }

  if (isEmpty) {
    return (
      <EmptyState
        icon={<PackageSearch className="h-12 w-12" />}
        title="No skills found"
        description="Try adjusting your search or browse the Discover page to find skills."
      />
    );
  }

  return (
    <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4">
      {renderCard
        ? skills.map((skill) => (
            <React.Fragment key={skill.id}>
              {renderCard(skill)}
            </React.Fragment>
          ))
        : skills.map((skill) => <SkillCard key={skill.id} skill={skill} />)}
    </div>
  );
}

export { SkillGrid };
