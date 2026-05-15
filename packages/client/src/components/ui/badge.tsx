import * as React from 'react';
import { cva, type VariantProps } from 'class-variance-authority';
import { cn } from '../../lib/utils';

const badgeVariants = cva(
  'inline-flex items-center rounded-full border px-2.5 py-0.5 text-xs font-semibold transition-colors',
  {
    variants: {
      variant: {
        default:
          'border-transparent bg-slate-900 dark:bg-slate-100 text-white dark:text-slate-900',
        success:
          'border-transparent bg-green-100 dark:bg-green-900 text-green-800 dark:text-green-100',
        warning:
          'border-transparent bg-yellow-100 dark:bg-yellow-900 text-yellow-800 dark:text-yellow-100',
        danger:
          'border-transparent bg-red-100 dark:bg-red-900 text-red-800 dark:text-red-100',
        info:
          'border-transparent bg-blue-100 dark:bg-blue-900 text-blue-800 dark:text-blue-100',
        outline: 'text-slate-600 dark:text-slate-400',
      },
    },
    defaultVariants: {
      variant: 'default',
    },
  },
);

export interface BadgeProps
  extends React.HTMLAttributes<HTMLDivElement>,
    VariantProps<typeof badgeVariants> {}

function Badge({ className, variant, ...props }: BadgeProps) {
  return (
    <div className={cn(badgeVariants({ variant }), className)} {...props} />
  );
}

export { Badge, badgeVariants };
