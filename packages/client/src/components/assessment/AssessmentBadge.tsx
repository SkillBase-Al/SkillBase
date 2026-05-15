import * as React from 'react';
import { cn } from '../../lib/utils';

interface AssessmentBadgeProps {
  score: number;
  label?: string;
  className?: string;
}

function AssessmentBadge({ score, label, className }: AssessmentBadgeProps) {
  const colorClass =
    score >= 80
      ? 'bg-green-100 dark:bg-green-900 text-green-700 dark:text-green-300'
      : score >= 50
        ? 'bg-yellow-100 dark:bg-yellow-900 text-yellow-700 dark:text-yellow-300'
        : 'bg-red-100 dark:bg-red-900 text-red-700 dark:text-red-300';

  return (
    <span
      className={cn(
        'inline-flex items-center gap-1 rounded-full px-2 py-0.5 text-xs font-semibold',
        colorClass,
        className,
      )}
    >
      {label && <span>{label}</span>}
      <span>{score}</span>
    </span>
  );
}

export { AssessmentBadge };
