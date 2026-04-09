import { observer } from "mobx-react-lite";
import { useAppViewmodel } from "@/app.viewmodel";
import { RunSummaryCard } from "@/components/organisms/run-summary-card";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { useDashboardViewmodel } from "../dashboard.viewmodel";

export const DashboardRuns = observer(() => {
  const viewmodel = useDashboardViewmodel();
  const appViewmodel = useAppViewmodel();

  return (
    <Card>
      <CardHeader>
        <CardTitle>Recent runs</CardTitle>
        <CardDescription>
          The latest execution trail across the workspace.
        </CardDescription>
      </CardHeader>
      <CardContent className="grid grid-cols-1 gap-4 md:grid-cols-2 lg:grid-cols-4">
        {viewmodel.recentRuns.length === 0 && (
          <p className="text-sm text-muted-foreground">
            No runs yet. Trigger one from the runs page or assign an issue.
          </p>
        )}
        {viewmodel.recentRuns.map((run) => (
          <RunSummaryCard
            key={run.id}
            latestActivity={appViewmodel.runs.latestActivity(run.id)}
            run={run}
          />
        ))}
      </CardContent>
    </Card>
  );
});
