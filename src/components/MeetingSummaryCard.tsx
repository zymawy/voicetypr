import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import type { MeetingSummary } from "@/types/meetings";
import { CheckSquare, Flag, Lightbulb } from "lucide-react";

interface MeetingSummaryCardProps {
  summary: MeetingSummary;
}

interface SummaryListProps {
  title: string;
  Icon: typeof Lightbulb;
  items: string[];
  emptyText: string;
}

function SummaryList({ title, Icon, items, emptyText }: SummaryListProps) {
  return (
    <Card>
      <CardHeader className="pb-2">
        <CardTitle className="text-sm font-semibold flex items-center gap-2">
          <Icon className="h-4 w-4 text-muted-foreground" />
          {title}
        </CardTitle>
      </CardHeader>
      <CardContent>
        {items.length === 0 ? (
          <p className="text-xs text-muted-foreground italic">{emptyText}</p>
        ) : (
          <ul className="text-sm space-y-1.5 list-disc list-inside marker:text-muted-foreground">
            {items.map((item, i) => (
              <li key={i} className="leading-relaxed">
                {item}
              </li>
            ))}
          </ul>
        )}
      </CardContent>
    </Card>
  );
}

export function MeetingSummaryCard({ summary }: MeetingSummaryCardProps) {
  return (
    <div className="space-y-4">
      <div className="grid gap-3 md:grid-cols-3">
        <SummaryList
          title="Key Points"
          Icon={Lightbulb}
          items={summary.key_points}
          emptyText="No key points captured"
        />
        <SummaryList
          title="Action Items"
          Icon={CheckSquare}
          items={summary.action_items}
          emptyText="No action items"
        />
        <SummaryList
          title="Decisions"
          Icon={Flag}
          items={summary.decisions}
          emptyText="No decisions"
        />
      </div>
      {summary.raw && (
        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="text-sm font-semibold">Full Summary</CardTitle>
          </CardHeader>
          <CardContent>
            <pre className="text-xs whitespace-pre-wrap leading-relaxed text-foreground/90 font-sans">
              {summary.raw}
            </pre>
          </CardContent>
        </Card>
      )}
    </div>
  );
}
