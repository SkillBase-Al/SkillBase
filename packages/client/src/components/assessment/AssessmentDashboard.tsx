import * as React from 'react';
import {
  CheckCircle2,
  AlertTriangle,
  XCircle,
  FileText,
  Shield,
  BarChart3,
} from 'lucide-react';
import { Card, CardContent } from '../ui/card';
import { cn } from '../../lib/utils';
import type { AssessmentSummary } from '../../types/assessment';
import { LoadingSkeleton } from '../shared/LoadingSkeleton';
import { ErrorState } from '../shared/ErrorState';
import { EmptyState } from '../shared/EmptyState';

interface AssessmentDashboardProps {
  summary: AssessmentSummary | null;
  isLoading: boolean;
  error: string | null;
  onRetry?: () => void;
}

function AssessmentDashboard({
  summary,
  isLoading,
  error,
  onRetry,
}: AssessmentDashboardProps) {
  if (isLoading) {
    return <LoadingSkeleton variant="detail" />;
  }

  if (error) {
    return <ErrorState message={error} onRetry={onRetry} />;
  }

  if (!summary) {
    return (
      <EmptyState
        icon={<BarChart3 className="h-12 w-12" />}
        title="No assessments yet"
        description="Run an assessment on your installed skills to see results here."
      />
    );
  }

  const stats = [
    {
      label: 'Total Assessed',
      value: summary.totalAssessed,
      icon: FileText,
      color: 'text-blue-600 dark:text-blue-400',
      bg: 'bg-blue-100 dark:bg-blue-900',
    },
    {
      label: 'Format Passed',
      value: summary.passedFormat,
      icon: CheckCircle2,
      color: 'text-green-600 dark:text-green-400',
      bg: 'bg-green-100 dark:bg-green-900',
    },
    {
      label: 'Security Passed',
      value: summary.passedSecurity,
      icon: Shield,
      color: 'text-purple-600 dark:text-purple-400',
      bg: 'bg-purple-100 dark:bg-purple-900',
    },
    {
      label: 'Total Issues',
      value: summary.totalIssues,
      icon: AlertTriangle,
      color: 'text-red-600 dark:text-red-400',
      bg: 'bg-red-100 dark:bg-red-900',
    },
  ];

  return (
    <div className="space-y-6">
      {/* Stats Cards */}
      <div className="grid grid-cols-2 lg:grid-cols-4 gap-4">
        {stats.map((stat) => (
          <Card key={stat.label}>
            <CardContent className="p-4">
              <div className="flex items-center gap-3">
                <div className={cn('p-2 rounded-lg', stat.bg)}>
                  <stat.icon className={cn('h-5 w-5', stat.color)} />
                </div>
                <div>
                  <p className="text-xs text-slate-500 dark:text-slate-400">
                    {stat.label}
                  </p>
                  <p className="text-xl font-bold text-slate-800 dark:text-slate-200">
                    {stat.value}
                  </p>
                </div>
              </div>
            </CardContent>
          </Card>
        ))}
      </div>

      {/* Average Scores */}
      <div className="grid grid-cols-2 gap-4">
        <Card>
          <CardContent className="p-4">
            <div className="flex items-center gap-2 mb-2">
              <FileText className="h-4 w-4 text-slate-500" />
              <span className="text-sm font-medium text-slate-700 dark:text-slate-300">
                Average Format Score
              </span>
            </div>
            <div className="flex items-center gap-2">
              <div className="flex-1 h-2 rounded-full bg-slate-200 dark:bg-slate-700">
                <div
                  className={cn(
                    'h-2 rounded-full transition-all',
                    summary.averageFormatScore >= 80
                      ? 'bg-green-500'
                      : summary.averageFormatScore >= 50
                        ? 'bg-yellow-500'
                        : 'bg-red-500',
                  )}
                  style={{ width: `${summary.averageFormatScore}%` }}
                />
              </div>
              <span className="text-sm font-semibold text-slate-700 dark:text-slate-300 min-w-[3ch] text-right">
                {summary.averageFormatScore.toFixed(0)}
              </span>
            </div>
          </CardContent>
        </Card>
        <Card>
          <CardContent className="p-4">
            <div className="flex items-center gap-2 mb-2">
              <Shield className="h-4 w-4 text-slate-500" />
              <span className="text-sm font-medium text-slate-700 dark:text-slate-300">
                Average Security Score
              </span>
            </div>
            <div className="flex items-center gap-2">
              <div className="flex-1 h-2 rounded-full bg-slate-200 dark:bg-slate-700">
                <div
                  className={cn(
                    'h-2 rounded-full transition-all',
                    summary.averageSecurityScore >= 80
                      ? 'bg-green-500'
                      : summary.averageSecurityScore >= 50
                        ? 'bg-yellow-500'
                        : 'bg-red-500',
                  )}
                  style={{ width: `${summary.averageSecurityScore}%` }}
                />
              </div>
              <span className="text-sm font-semibold text-slate-700 dark:text-slate-300 min-w-[3ch] text-right">
                {summary.averageSecurityScore.toFixed(0)}
              </span>
            </div>
          </CardContent>
        </Card>
      </div>

      {/* Results Table */}
      {summary.results.length > 0 && (
        <Card>
          <CardContent className="p-4">
            <h4 className="text-sm font-medium text-slate-700 dark:text-slate-300 mb-3">
              Detailed Results
            </h4>
            <div className="space-y-2">
              {summary.results.map((result) => (
                <div
                  key={result.skillId}
                  className="flex items-center justify-between p-2 rounded-lg bg-slate-50 dark:bg-slate-800"
                >
                  <div className="flex items-center gap-2 min-w-0">
                    {result.passed ? (
                      <CheckCircle2 className="h-4 w-4 text-green-500 flex-shrink-0" />
                    ) : (
                      <XCircle className="h-4 w-4 text-red-500 flex-shrink-0" />
                    )}
                    <span className="text-sm text-slate-700 dark:text-slate-300 truncate">
                      {result.skillName}
                    </span>
                  </div>
                  <div className="flex items-center gap-3 flex-shrink-0">
                    <span className="text-xs text-slate-500">
                      F: {result.formatScore}
                    </span>
                    <span className="text-xs text-slate-500">
                      S: {result.securityScore}
                    </span>
                  </div>
                </div>
              ))}
            </div>
          </CardContent>
        </Card>
      )}
    </div>
  );
}

export { AssessmentDashboard };
