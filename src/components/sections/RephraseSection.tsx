import { RephraseSettings } from "@/components/RephraseSettings";
import { Button } from "@/components/ui/button";
import { ScrollArea } from "@/components/ui/scroll-area";
import type { RephraseSettings as RephraseSettingsType, RephraseStyle } from "@/types/ai";
import { invoke } from "@tauri-apps/api/core";
import { useEffect, useState, useRef, useCallback } from "react";
import { toast } from "sonner";
import { Info, Play } from "lucide-react";
import { getErrorMessage } from "@/utils/error";

interface AISettingsState {
  enabled: boolean;
  provider: string;
  model: string;
  hasApiKey: boolean;
}

export function RephraseSection() {
  const [rephraseSettings, setRephraseSettings] = useState<{
    style: RephraseStyle;
    customInstructions?: string;
    hotkey: string;
  }>({
    style: "Professional",
    customInstructions: undefined,
    hotkey: "CommandOrControl+Shift+.",
  });

  const [aiEnabled, setAiEnabled] = useState(false);
  const [isTesting, setIsTesting] = useState(false);
  const [settingsLoaded, setSettingsLoaded] = useState(false);
  const customInstructionsTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  const loadSettings = useCallback(async () => {
    try {
      const [rephrase, aiSettings] = await Promise.all([
        invoke<RephraseSettingsType>("get_rephrase_settings"),
        invoke<AISettingsState>("get_ai_settings"),
      ]);

      setRephraseSettings({
        style: rephrase.style,
        customInstructions: rephrase.custom_instructions,
        hotkey: rephrase.hotkey,
      });
      setAiEnabled(aiSettings.enabled);
    } catch (error) {
      const message = getErrorMessage(error, "Failed to load rephrase settings");
      toast.error(message);
    }
  }, []);

  useEffect(() => {
    if (!settingsLoaded) {
      loadSettings();
      setSettingsLoaded(true);
    }
  }, [settingsLoaded, loadSettings]);

  // Cleanup debounce timer
  useEffect(() => {
    return () => {
      if (customInstructionsTimerRef.current) {
        clearTimeout(customInstructionsTimerRef.current);
      }
    };
  }, []);

  const handleSettingsChange = async (newSettings: typeof rephraseSettings) => {
    const prevSettings = rephraseSettings;
    setRephraseSettings(newSettings);

    // Debounce custom instructions saves
    const instructionsChanged = newSettings.customInstructions !== prevSettings.customInstructions;
    if (instructionsChanged) {
      if (customInstructionsTimerRef.current) {
        clearTimeout(customInstructionsTimerRef.current);
      }
      customInstructionsTimerRef.current = setTimeout(async () => {
        try {
          await invoke("update_rephrase_settings", {
            style: newSettings.style,
            customInstructions: newSettings.customInstructions || null,
          });
        } catch (error) {
          const message = getErrorMessage(error, "Failed to save rephrase settings");
          toast.error(message);
        }
      }, 500);
      return;
    }

    // Save style changes immediately
    if (newSettings.style !== prevSettings.style) {
      try {
        await invoke("update_rephrase_settings", {
          style: newSettings.style,
          customInstructions: newSettings.customInstructions || null,
        });
      } catch (error) {
        const message = getErrorMessage(error, "Failed to save rephrase settings");
        toast.error(message);
      }
    }
  };

  const handleTestRephrase = async () => {
    setIsTesting(true);
    try {
      await invoke("rephrase_selected_text", {
        style: rephraseSettings.style,
        customInstructions: rephraseSettings.customInstructions || null,
      });
      toast.success("Rephrase triggered. Select text in any app first.");
    } catch (error) {
      const message = getErrorMessage(error, "Failed to rephrase text");
      toast.error(message);
    } finally {
      setIsTesting(false);
    }
  };

  return (
    <div className="h-full flex flex-col">
      {/* Header */}
      <div className="px-6 py-4 border-b border-border/40">
        <div className="flex items-center justify-between">
          <div>
            <h1 className="text-2xl font-semibold">Rephrase</h1>
            <p className="text-sm text-muted-foreground mt-1">
              Highlight text in any app, press the hotkey, and have it rephrased by AI
            </p>
          </div>
          <Button
            size="sm"
            className="gap-2"
            onClick={handleTestRephrase}
            disabled={!aiEnabled || isTesting}
          >
            <Play className="h-4 w-4" />
            {isTesting ? "Testing..." : "Test Rephrase"}
          </Button>
        </div>
      </div>

      <ScrollArea className="flex-1">
        <div className="p-6 space-y-6">
          {/* AI not enabled warning */}
          {!aiEnabled && (
            <div className="rounded-lg border border-border/50 bg-card p-4">
              <div className="flex items-start gap-3">
                <div className="p-1.5 rounded-md bg-amber-500/10">
                  <Info className="h-4 w-4 text-amber-500" />
                </div>
                <div className="space-y-2 flex-1">
                  <h3 className="font-medium text-sm">AI Enhancement Required</h3>
                  <p className="text-sm text-muted-foreground">
                    Rephrase requires AI formatting to be enabled. Go to the Formatting section, configure an AI provider, and toggle AI Formatting on.
                  </p>
                </div>
              </div>
            </div>
          )}

          {/* Rephrase Style & Options */}
          <div className="space-y-4">
            <div className="flex items-center gap-2">
              <h2 className="text-base font-semibold">Rephrase Options</h2>
              <div className="h-px bg-border/50 flex-1" />
            </div>

            <RephraseSettings
              settings={rephraseSettings}
              onSettingsChange={handleSettingsChange}
              disabled={!aiEnabled}
            />
          </div>
        </div>
      </ScrollArea>
    </div>
  );
}
