import { Link } from "@tanstack/react-router";
import { ChevronRightIcon } from "lucide-react";
import { observer } from "mobx-react-lite";
import { RunUsageSummary } from "@/components/pages/run/components/run-usage-summary";
import { formatRunTime, formatRunTrigger } from "@/lib/runs";
import { AppModel } from "@/models/app.model";
import type { RunSummaryModel } from "@/models/run-summary.model";
import { RunStatusChip } from "./run-status-chip";
import { cn } from "@/lib/utils";

interface RunSummaryCardProps {
  run: RunSummaryModel;
  latestActivity?: string | null;
  className?: string;
}

export const RunSummaryCard = observer(
  ({ run, latestActivity, className }: RunSummaryCardProps) => {
    return (
      <Link
        params={{ runId: run.id }}
        to="/runs/$runId"
        className={cn(
          "group flex items-start justify-between gap-4 rounded-sm border border-foreground/10 bg-card px-4 py-3 transition-colors hover:bg-muted/50",
          className,
        )}
      >
        <div className="min-w-0 flex-1 space-y-2">
          <div className="flex flex-wrap items-center gap-2">
            <p className="font-medium">
              {AppModel.instance.resolveEmployeeName(run.employeeId) ??
                "Unknown employee"}
            </p>
            <RunStatusChip status={run.status} />
          </div>
          <p className="text-xs text-muted-foreground">
            {formatRunTrigger(run.trigger)} ·{" "}
            {formatRunTime(run.startedAt ?? run.createdAt)}
          </p>
          <RunUsageSummary compact usage={run.usage} />
          <p className="truncate text-sm text-muted-foreground">
            {latestActivity?.trim() || "Open run details"}
          </p>
        </div>
        <ChevronRightIcon className="mt-0.5 size-4 shrink-0 text-muted-foreground transition-transform group-hover:translate-x-0.5" />
      </Link>
    );
  },
);
