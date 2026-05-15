import * as React from 'react';
import {
  GitCompareArrows,
  Trash2,
  Eye,
  Percent,
} from 'lucide-react';
import { Card, CardContent } from '../ui/card';
import { Button } from '../ui/button';
import { Badge } from '../ui/badge';
import { cn } from '../../lib/utils';
import type { DedupGroup } from '../../types/dedup';

interface DedupGroupCardProps {
  group: DedupGroup;
  onDelete: (skillId: string) => void;
  onCompare: () => void;
}

function DedupGroupCard({ group, onDelete, onCompare }: DedupGroupCardProps) {
  const similarityPercent = (group.similarity * 100).toFixed(0);

  return (
    <Card>
      <CardContent className="p-4">
        {/* Header */}
        <div className="flex items-center justify-between mb-3">
          <div className="flex items-center gap-2">
            <GitCompareArrows className="h-4 w-4 text-slate-500" />
            <span className="text-sm font-medium text-slate-700 dark:text-slate-300">
              Duplicate Group
            </span>
          </div>
          <Badge
            variant={
              group.similarity >= 0.9
                ? 'danger'
                : group.similarity >= 0.7
                  ? 'warning'
                  : 'info'
            }
            className="text-xs"
          >
            <Percent className="h-3 w-3 mr-1" />
            {similarityPercent}% match
          </Badge>
        </div>

        {/* Skill cards side by side */}
        <div className="grid grid-cols-2 gap-3 mb-3">
          {group.skills.map((skill) => (
            <div
              key={skill.id}
              className="rounded-lg border border-slate-200 dark:border-slate-700 p-3 bg-slate-50 dark:bg-slate-800"
            >
              <h4 className="text-sm font-medium text-slate-800 dark:text-slate-200 truncate mb-1">
                {skill.name}
              </h4>
              <p className="text-xs text-slate-500 dark:text-slate-400 truncate mb-1">
                {skill.description}
              </p>
              <p className="text-[10px] text-slate-400 truncate">
                {skill.path}
              </p>
              <p className="text-[10px] text-slate-400">
                {skill.version ? `v${skill.version}` : ''}
              </p>
            </div>
          ))}
        </div>

        {/* Actions */}
        <div className="flex items-center gap-2">
          <Button
            variant="outline"
            size="sm"
            onClick={onCompare}
          >
            <Eye className="h-3 w-3 mr-1" />
            Compare
          </Button>
          {group.skills.slice(1).map((skill) => (
            <Button
              key={skill.id}
              variant="ghost"
              size="sm"
              onClick={() => onDelete(skill.id)}
              className="text-red-500 hover:text-red-700 hover:bg-red-50 dark:hover:bg-red-950"
            >
              <Trash2 className="h-3 w-3 mr-1" />
              Remove
            </Button>
          ))}
        </div>
      </CardContent>
    </Card>
  );
}

export { DedupGroupCard };
