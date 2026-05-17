import * as React from 'react';
import { Package, Star, Shield } from 'lucide-react';
import { Card, CardContent, CardFooter } from '../ui/card';
import { Badge } from '../ui/badge';
import { Switch } from '../ui/switch';
import { cn } from '../../lib/utils';
import type { InstalledSkill, MarketSkill } from '../../types/skill';

type SkillCardSkill = InstalledSkill | MarketSkill;

interface SkillCardProps {
  skill: SkillCardSkill;
  enabled?: boolean;
  onToggle?: (enabled: boolean) => void;
  onClick?: () => void;
  showInstallButton?: boolean;
  onInstall?: () => void;
  agentNames?: string[];
  className?: string;
}

function isMarketSkill(skill: SkillCardSkill): skill is MarketSkill {
  return 'downloads' in skill && 'rating' in skill;
}

function SkillCard({
  skill,
  enabled,
  onToggle,
  onClick,
  showInstallButton,
  onInstall,
  agentNames,
  className,
}: SkillCardProps) {
  return (
    <Card
      className={cn(
        'cursor-pointer transition-all hover:shadow-md hover:border-slate-300 dark:hover:border-slate-600',
        className,
      )}
      onClick={onClick}
    >
      <CardContent className="p-4">
        <div className="flex items-start justify-between mb-2">
          <div className="flex items-center gap-2 min-w-0">
            <div className="flex items-center justify-center h-8 w-8 rounded-md bg-blue-100 dark:bg-blue-900 flex-shrink-0">
              <Package className="h-4 w-4 text-blue-600 dark:text-blue-300" />
            </div>
            <div className="min-w-0">
              <h3 className="font-medium text-sm text-slate-800 dark:text-slate-200 truncate">
                {skill.name}
              </h3>
              <p className="text-xs text-slate-400">
                {skill.version ? `v${skill.version}` : ''}
              </p>
            </div>
          </div>

          {onToggle && enabled !== undefined && (
            <Switch
              checked={enabled}
              onCheckedChange={onToggle}
              onClick={(e: React.MouseEvent) => e.stopPropagation()}
            />
          )}

          {showInstallButton && onInstall && (
            <button
              onClick={(e) => {
                e.stopPropagation();
                onInstall();
              }}
              className="text-xs bg-blue-600 text-white px-3 py-1 rounded-md hover:bg-blue-700 transition-colors"
            >
              Install
            </button>
          )}
        </div>

        <p className="text-xs text-slate-500 dark:text-slate-400 line-clamp-2 mb-3">
          {skill.description}
        </p>

        <div className="flex flex-wrap gap-1.5 mb-2">
          {'source' in skill && skill.source && skill.source !== 'github' && (
            <Badge variant="outline" className="text-[10px] px-1.5 py-0 border-blue-200 text-blue-700">
              {skill.source.split('/').pop()}
            </Badge>
          )}
          {skill.tags.slice(0, 3).map((tag) => (
            <Badge key={tag} variant="outline" className="text-[10px] px-1.5 py-0">
              {tag}
            </Badge>
          ))}
          {skill.tags.length > 3 && (
            <Badge variant="outline" className="text-[10px] px-1.5 py-0">
              +{skill.tags.length - 3}
            </Badge>
          )}
        </div>

        {isMarketSkill(skill) && (
          <div className="flex items-center gap-3 text-xs text-slate-400">
            <span className="flex items-center gap-1">
              <Star className="h-3 w-3 text-yellow-500" />
              {skill.rating.toFixed(1)}
            </span>
            <span>{skill.downloads} downloads</span>
          </div>
        )}

        {'formatScore' in skill && skill.formatScore != null && (
          <div className="flex items-center gap-2 mt-1">
            <span
              className={cn(
                'text-xs px-1.5 py-0.5 rounded font-medium',
                skill.formatScore >= 80
                  ? 'bg-green-100 text-green-700 dark:bg-green-900 dark:text-green-300'
                  : skill.formatScore >= 50
                    ? 'bg-yellow-100 text-yellow-700 dark:bg-yellow-900 dark:text-yellow-300'
                    : 'bg-red-100 text-red-700 dark:bg-red-900 dark:text-red-300',
              )}
            >
              F: {skill.formatScore}
            </span>
            {'safetyLevel' in skill && skill.safetyLevel != null && (
              <span
                className={cn(
                  'text-xs px-1.5 py-0.5 rounded font-medium',
                  skill.safetyLevel === 'Safe'
                    ? 'bg-green-100 text-green-700 dark:bg-green-900 dark:text-green-300'
                    : skill.safetyLevel === 'Warning'
                      ? 'bg-yellow-100 text-yellow-700 dark:bg-yellow-900 dark:text-yellow-300'
                      : 'bg-red-100 text-red-700 dark:bg-red-900 dark:text-red-300',
                )}
              >
                {skill.safetyLevel}
              </span>
            )}
            {'hasSecurityIssues' in skill && skill.hasSecurityIssues && (
              <Shield className="h-3 w-3 text-red-500" />
            )}
          </div>
        )}
      </CardContent>

      {'agentCount' in skill && skill.agentCount > 0 && (
        <CardFooter className="px-4 py-2 border-t border-slate-100 dark:border-slate-800 flex flex-wrap items-center gap-x-1">
          <span className="text-xs text-slate-400 whitespace-nowrap">
            {skill.agentCount} agent(s)
          </span>
          {agentNames && agentNames.length > 0 && (
            <span className="text-xs text-slate-400 truncate min-w-0">
              <span className="mx-0.5">·</span>
              {agentNames.slice(0, 4).join(', ')}
              {agentNames.length > 4 && (
                <span className="text-slate-400 ml-0.5">...</span>
              )}
            </span>
          )}
        </CardFooter>
      )}
    </Card>
  );
}

export { SkillCard };
export type { SkillCardProps };
