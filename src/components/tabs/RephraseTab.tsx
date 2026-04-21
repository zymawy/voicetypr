import { useEffect } from "react";
import { toast } from "sonner";
import { RephraseSection } from "../sections/RephraseSection";
import { useEventCoordinator } from "@/hooks/useEventCoordinator";

export function RephraseTab() {
  const { registerEvent } = useEventCoordinator("main");

  useEffect(() => {
    const init = async () => {
      try {
        registerEvent("rephrase-error", (event) => {
          toast.error(event.payload as string, {
            description: "Check your AI settings in the Formatting section",
          });
        });
      } catch (error) {
        // Initialization error handled silently
      }
    };

    init();
  }, [registerEvent]);

  return <RephraseSection />;
}
