import { MeetingsSection } from "@/components/sections/MeetingsSection";
import type { MeetingErrorEvent } from "@/types/meetings";
import { listen } from "@tauri-apps/api/event";
import { useEffect } from "react";
import { toast } from "sonner";

export function MeetingsTab() {
  useEffect(() => {
    let unlisten: (() => void) | null = null;
    listen<MeetingErrorEvent>("meeting:error", (event) => {
      const msg = event.payload.message;
      if (event.payload.kind === "screen_recording") {
        toast.error("Screen Recording permission needed", { description: msg });
      } else if (event.payload.kind === "microphone") {
        toast.error("Microphone permission needed", { description: msg });
      } else {
        toast.error("Meeting error", { description: msg });
      }
    }).then((u) => {
      unlisten = u;
    });
    return () => {
      if (unlisten) unlisten();
    };
  }, []);

  return <MeetingsSection />;
}
