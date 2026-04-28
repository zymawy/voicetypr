import { MeetingSummaryCard } from "@/components/MeetingSummaryCard";
import { MeetingTranscript } from "@/components/MeetingTranscript";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { Input } from "@/components/ui/input";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { useLiveMeeting } from "@/hooks/useLiveMeeting";
import { formatDuration, formatRelativeDate } from "@/lib/meeting-format";
import { cn } from "@/lib/utils";
import type {
  Meeting,
  MeetingIndexEntry,
  MeetingSummary,
} from "@/types/meetings";
import { invoke } from "@tauri-apps/api/core";
import { ArrowLeft, FileText, Loader2, Mic, Sparkles, Square, Trash2 } from "lucide-react";
import { useCallback, useEffect, useState } from "react";
import { toast } from "sonner";

type ViewMode =
  | { kind: "list" }
  | { kind: "recording"; meetingId: string }
  | { kind: "detail"; meetingId: string };

export function MeetingsSection() {
  const [view, setView] = useState<ViewMode>({ kind: "list" });
  const [meetings, setMeetings] = useState<MeetingIndexEntry[]>([]);
  const [isLoading, setIsLoading] = useState(false);
  const [starting, setStarting] = useState(false);

  const refreshList = useCallback(async () => {
    try {
      setIsLoading(true);
      const list = await invoke<MeetingIndexEntry[]>("list_meetings");
      setMeetings(list);
    } catch (e) {
      toast.error(`Failed to load meetings: ${e}`);
    } finally {
      setIsLoading(false);
    }
  }, []);

  useEffect(() => {
    refreshList();
  }, [refreshList]);

  const startMeeting = useCallback(async () => {
    try {
      setStarting(true);
      const id = await invoke<string>("start_meeting", { title: null });
      setView({ kind: "recording", meetingId: id });
    } catch (e) {
      toast.error(`Failed to start meeting: ${e}`);
    } finally {
      setStarting(false);
    }
  }, []);

  if (view.kind === "recording") {
    return (
      <RecordingView
        meetingId={view.meetingId}
        onComplete={async () => {
          await refreshList();
          setView({ kind: "detail", meetingId: view.meetingId });
        }}
      />
    );
  }
  if (view.kind === "detail") {
    return (
      <DetailView
        meetingId={view.meetingId}
        onBack={() => {
          refreshList();
          setView({ kind: "list" });
        }}
        onDeleted={() => {
          refreshList();
          setView({ kind: "list" });
        }}
      />
    );
  }

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-lg font-semibold">Meetings</h2>
          <p className="text-sm text-muted-foreground">
            Record meetings with system + microphone audio. Get a live transcript and an AI summary.
          </p>
        </div>
        <Button onClick={startMeeting} disabled={starting} size="lg">
          {starting ? <Loader2 className="h-4 w-4 mr-2 animate-spin" /> : <Mic className="h-4 w-4 mr-2" />}
          Start Meeting
        </Button>
      </div>

      <ScrollArea className="rounded-md border max-h-[60vh]">
        <div className="divide-y">
          {meetings.length === 0 && !isLoading && (
            <div className="p-8 text-center text-sm text-muted-foreground">
              No meetings yet. Start one to capture transcript + summary.
            </div>
          )}
          {meetings.map((m) => (
            <button
              key={m.id}
              onClick={() => setView({ kind: "detail", meetingId: m.id })}
              className="w-full text-left p-4 hover:bg-accent/40 transition-colors flex items-center gap-3"
            >
              <FileText className="h-4 w-4 text-muted-foreground shrink-0" />
              <div className="flex-1 min-w-0">
                <div className="font-medium truncate">{m.title}</div>
                <div className="text-xs text-muted-foreground">
                  {formatRelativeDate(m.started_at)} · {formatDuration(m.duration_seconds)}
                  {m.has_summary && (
                    <span className="ml-2 inline-flex items-center gap-1 text-emerald-600 dark:text-emerald-400">
                      <Sparkles className="h-3 w-3" /> summary
                    </span>
                  )}
                </div>
              </div>
            </button>
          ))}
        </div>
      </ScrollArea>
    </div>
  );
}

function RecordingView({
  meetingId,
  onComplete,
}: {
  meetingId: string;
  onComplete: () => Promise<void> | void;
}) {
  const { segments, status, elapsedSeconds } = useLiveMeeting(meetingId);
  const [stopping, setStopping] = useState(false);

  const stop = async () => {
    if (stopping) return;
    setStopping(true);
    try {
      await invoke<Meeting>("stop_meeting", { meetingId });
      await onComplete();
    } catch (e) {
      toast.error(`Stop failed: ${e}`);
      setStopping(false);
    }
  };

  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between">
        <div>
          <div className="text-3xl font-mono tabular-nums">{formatDuration(elapsedSeconds)}</div>
          <div className="flex items-center gap-2 mt-1">
            <span
              className={cn(
                "inline-flex items-center gap-1.5 px-2 py-0.5 rounded-md text-xs font-medium",
                status === "recording" && "bg-red-500/10 text-red-600 dark:text-red-400",
                status === "starting" && "bg-amber-500/10 text-amber-600 dark:text-amber-400",
                (status === "stopping" || status === "transcribing" || status === "summarizing") &&
                  "bg-blue-500/10 text-blue-600 dark:text-blue-400",
                status === "error" && "bg-destructive/10 text-destructive",
                status === "idle" && "bg-muted text-muted-foreground",
              )}
            >
              {status === "recording" && (
                <span className="relative flex h-1.5 w-1.5">
                  <span className="animate-ping absolute inline-flex h-full w-full rounded-full bg-red-500 opacity-75" />
                  <span className="relative inline-flex rounded-full h-1.5 w-1.5 bg-red-500" />
                </span>
              )}
              {status[0].toUpperCase() + status.slice(1)}
            </span>
          </div>
        </div>
        <Button variant="destructive" size="lg" onClick={stop} disabled={stopping}>
          {stopping ? <Loader2 className="h-4 w-4 mr-2 animate-spin" /> : <Square className="h-4 w-4 mr-2" />}
          Stop Meeting
        </Button>
      </div>

      <MeetingTranscript
        segments={segments}
        autoScroll
        className="h-[55vh]"
        emptyText="Listening… first transcript chunk in ~15 seconds."
      />
    </div>
  );
}

function DetailView({
  meetingId,
  onBack,
  onDeleted,
}: {
  meetingId: string;
  onBack: () => void;
  onDeleted: () => void;
}) {
  const [meeting, setMeeting] = useState<Meeting | null>(null);
  const [editingTitle, setEditingTitle] = useState(false);
  const [titleDraft, setTitleDraft] = useState("");
  const [summarizing, setSummarizing] = useState(false);
  const [confirmDelete, setConfirmDelete] = useState(false);

  const load = useCallback(async () => {
    try {
      const m = await invoke<Meeting>("get_meeting", { meetingId });
      setMeeting(m);
      setTitleDraft(m.title);
    } catch (e) {
      toast.error(`Failed to load meeting: ${e}`);
    }
  }, [meetingId]);

  useEffect(() => {
    load();
  }, [load]);

  const summarize = async () => {
    setSummarizing(true);
    try {
      const summary = await invoke<MeetingSummary>("summarize_meeting", { meetingId });
      setMeeting((prev) => (prev ? { ...prev, summary } : prev));
      toast.success("Summary generated");
    } catch (e) {
      toast.error(`Summary failed: ${e}`);
    } finally {
      setSummarizing(false);
    }
  };

  const saveTitle = async () => {
    if (!titleDraft.trim()) {
      setTitleDraft(meeting?.title ?? "");
      setEditingTitle(false);
      return;
    }
    try {
      const updated = await invoke<Meeting>("rename_meeting", {
        meetingId,
        title: titleDraft.trim(),
      });
      setMeeting(updated);
      setEditingTitle(false);
    } catch (e) {
      toast.error(`Rename failed: ${e}`);
    }
  };

  const doDelete = async () => {
    try {
      await invoke<void>("delete_meeting", { meetingId });
      onDeleted();
    } catch (e) {
      toast.error(`Delete failed: ${e}`);
    }
  };

  const copy = async (text: string, label: string) => {
    try {
      await navigator.clipboard.writeText(text);
      toast.success(`${label} copied`);
    } catch {
      toast.error("Copy failed");
    }
  };

  if (!meeting) {
    return (
      <div className="flex items-center justify-center h-40 text-muted-foreground">
        <Loader2 className="h-5 w-5 animate-spin" />
      </div>
    );
  }

  const transcriptText = meeting.segments
    .map((s) => `${s.speaker === "me" ? "Me" : s.speaker === "them" ? "Them" : "Speaker"}: ${s.text}`)
    .join("\n");

  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between">
        <Button variant="ghost" size="sm" onClick={onBack} className="-ml-2">
          <ArrowLeft className="h-4 w-4 mr-1" /> Back
        </Button>
        <Button variant="ghost" size="icon" onClick={() => setConfirmDelete(true)}>
          <Trash2 className="h-4 w-4 text-destructive" />
        </Button>
      </div>

      <div>
        {editingTitle ? (
          <Input
            value={titleDraft}
            autoFocus
            onChange={(e) => setTitleDraft(e.target.value)}
            onBlur={saveTitle}
            onKeyDown={(e) => {
              if (e.key === "Enter") saveTitle();
              if (e.key === "Escape") {
                setTitleDraft(meeting.title);
                setEditingTitle(false);
              }
            }}
            className="text-xl font-semibold h-auto py-1"
          />
        ) : (
          <h2
            className="text-xl font-semibold cursor-text hover:bg-accent/30 rounded px-1 -mx-1"
            onClick={() => setEditingTitle(true)}
          >
            {meeting.title}
          </h2>
        )}
        <p className="text-sm text-muted-foreground">
          {formatRelativeDate(meeting.started_at)} · {formatDuration(meeting.duration_seconds)} ·{" "}
          {meeting.segments.length} segments
        </p>
      </div>

      <Tabs defaultValue="summary">
        <TabsList>
          <TabsTrigger value="summary">Summary</TabsTrigger>
          <TabsTrigger value="transcript">Transcript</TabsTrigger>
        </TabsList>

        <TabsContent value="summary" className="space-y-4">
          {meeting.summary ? (
            <>
              <div className="flex justify-end">
                <Button variant="outline" size="sm" onClick={() => copy(meeting.summary!.raw, "Summary")}>
                  Copy summary
                </Button>
              </div>
              <MeetingSummaryCard summary={meeting.summary} />
            </>
          ) : (
            <Card>
              <CardHeader>
                <CardTitle className="text-base">No summary yet</CardTitle>
              </CardHeader>
              <CardContent>
                <p className="text-sm text-muted-foreground mb-4">
                  Generate an AI summary with key points, action items, and decisions extracted from the
                  transcript.
                </p>
                <Button onClick={summarize} disabled={summarizing || meeting.segments.length === 0}>
                  {summarizing ? (
                    <Loader2 className="h-4 w-4 mr-2 animate-spin" />
                  ) : (
                    <Sparkles className="h-4 w-4 mr-2" />
                  )}
                  Generate Summary
                </Button>
                {meeting.segments.length === 0 && (
                  <p className="text-xs text-muted-foreground mt-2">
                    No transcript to summarize.
                  </p>
                )}
              </CardContent>
            </Card>
          )}
        </TabsContent>

        <TabsContent value="transcript" className="space-y-3">
          <div className="flex justify-end">
            <Button variant="outline" size="sm" onClick={() => copy(transcriptText, "Transcript")}>
              Copy transcript
            </Button>
          </div>
          <MeetingTranscript
            segments={meeting.segments}
            className="h-[55vh]"
            emptyText="No transcript captured."
          />
        </TabsContent>
      </Tabs>

      <Dialog open={confirmDelete} onOpenChange={setConfirmDelete}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>Delete this meeting?</DialogTitle>
            <DialogDescription>
              The transcript and summary will be permanently removed.
            </DialogDescription>
          </DialogHeader>
          <DialogFooter>
            <Button variant="outline" onClick={() => setConfirmDelete(false)}>
              Cancel
            </Button>
            <Button variant="destructive" onClick={doDelete}>
              Delete
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  );
}
