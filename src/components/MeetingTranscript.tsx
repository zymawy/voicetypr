import { ScrollArea } from "@/components/ui/scroll-area";
import { cn } from "@/lib/utils";
import { formatTimestampMs } from "@/lib/meeting-format";
import type { MeetingSegment } from "@/types/meetings";
import { useEffect, useRef } from "react";

interface MeetingTranscriptProps {
  segments: MeetingSegment[];
  autoScroll?: boolean;
  className?: string;
  emptyText?: string;
}

const speakerStyles: Record<string, string> = {
  me: "bg-blue-500/10 text-blue-700 dark:text-blue-300",
  them: "bg-purple-500/10 text-purple-700 dark:text-purple-300",
  mixed: "bg-zinc-500/10 text-zinc-700 dark:text-zinc-300",
};

const speakerLabels: Record<string, string> = {
  me: "Me",
  them: "Them",
  mixed: "Speaker",
};

export function MeetingTranscript({
  segments,
  autoScroll = false,
  className,
  emptyText = "Listening for speech…",
}: MeetingTranscriptProps) {
  const bottomRef = useRef<HTMLDivElement | null>(null);

  useEffect(() => {
    if (autoScroll && bottomRef.current) {
      bottomRef.current.scrollIntoView({ behavior: "smooth", block: "end" });
    }
  }, [segments, autoScroll]);

  return (
    <ScrollArea className={cn("rounded-md border bg-card", className)}>
      <div className="p-4 space-y-3">
        {segments.length === 0 && (
          <p className="text-sm text-muted-foreground italic">{emptyText}</p>
        )}
        {segments.map((seg, i) => (
          <div key={i} className="flex gap-3 items-start">
            <span
              className={cn(
                "shrink-0 inline-flex items-center px-2 py-0.5 rounded-md text-xs font-medium",
                speakerStyles[seg.speaker] ?? speakerStyles.mixed,
              )}
            >
              {speakerLabels[seg.speaker] ?? "Speaker"}
            </span>
            <div className="flex-1 min-w-0">
              <p className="text-sm leading-relaxed text-foreground whitespace-pre-wrap">
                {seg.text}
              </p>
              <span className="text-[10px] text-muted-foreground tabular-nums">
                {formatTimestampMs(seg.started_at_ms)}
              </span>
            </div>
          </div>
        ))}
        <div ref={bottomRef} />
      </div>
    </ScrollArea>
  );
}
