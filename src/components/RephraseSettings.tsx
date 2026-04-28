import { Button } from "@/components/ui/button";
import { Briefcase, Minimize2, Heart, CheckCircle, Expand } from "lucide-react";
import type { RephraseStyle } from "@/types/ai";

interface RephraseSettingsProps {
  settings: {
    style: RephraseStyle;
    customInstructions?: string;
    hotkey: string;
  };
  onSettingsChange: (settings: RephraseSettingsProps['settings']) => void;
  disabled?: boolean;
}

const ALL_STYLES: RephraseStyle[] = ["Professional", "Concise", "Friendly", "FixGrammar", "Elaborate"];

const styles = [
  { id: "Professional" as const, label: "Professional", icon: Briefcase, description: "Formal and polished tone" },
  { id: "Concise" as const, label: "Concise", icon: Minimize2, description: "Shorter and to the point" },
  { id: "Friendly" as const, label: "Friendly", icon: Heart, description: "Warm and approachable" },
  { id: "FixGrammar" as const, label: "Fix Grammar", icon: CheckCircle, description: "Correct grammar and spelling" },
  { id: "Elaborate" as const, label: "Elaborate", icon: Expand, description: "More detailed and descriptive" },
];

export function RephraseSettings({ settings, onSettingsChange, disabled = false }: RephraseSettingsProps) {
  const handleStyleChange = (style: string) => {
    if (ALL_STYLES.includes(style as RephraseStyle)) {
      onSettingsChange({
        ...settings,
        style: style as RephraseStyle,
      });
    }
  };

  return (
    <div className={`space-y-6 ${disabled ? 'opacity-50' : ''}`}>
      {/* Rephrase Style */}
      <div className="space-y-3">
        <div className="flex flex-wrap gap-2">
          {styles.map((style) => {
            const Icon = style.icon;
            const isSelected = settings.style === style.id;

            return (
              <Button
                key={style.id}
                variant={isSelected ? "default" : "outline"}
                size="sm"
                className={`gap-2 ${disabled ? 'cursor-not-allowed' : ''}`}
                onClick={() => !disabled && handleStyleChange(style.id)}
                disabled={disabled}
              >
                <Icon className="h-4 w-4" />
                {style.label}
              </Button>
            );
          })}
        </div>

        {/* Style description */}
        <p className="text-sm text-muted-foreground">
          {settings.style === "Professional" && "Rewrite in a formal, polished tone suitable for work communication"}
          {settings.style === "Concise" && "Shorten and tighten the text while preserving its meaning"}
          {settings.style === "Friendly" && "Rewrite in a warm, approachable, and conversational tone"}
          {settings.style === "FixGrammar" && "Fix grammar, spelling, and punctuation without changing the style"}
          {settings.style === "Elaborate" && "Expand and add detail to make the text more descriptive and thorough"}
        </p>
      </div>

      {/* Custom Instructions */}
      <div className="space-y-2">
        <label className="text-sm font-medium">Custom Instructions</label>
        <textarea
          className="w-full rounded-md border border-input bg-background px-3 py-2 text-sm ring-offset-background placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring disabled:cursor-not-allowed disabled:opacity-50 min-h-[80px] resize-y"
          placeholder="e.g., Keep it under 3 sentences, Use British English, Match my casual style..."
          value={settings.customInstructions || ""}
          onChange={(e) => !disabled && onSettingsChange({
            ...settings,
            customInstructions: e.target.value || undefined,
          })}
          disabled={disabled}
        />
        <p className="text-xs text-muted-foreground">
          Add personal preferences that apply when rephrasing selected text
        </p>
      </div>

      {/* Hotkey Configuration */}
      <div className="space-y-2">
        <label className="text-sm font-medium">Hotkey</label>
        <input
          type="text"
          className="w-full rounded-md border border-input bg-background px-3 py-2 text-sm ring-offset-background placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring disabled:cursor-not-allowed disabled:opacity-50"
          value={settings.hotkey}
          readOnly
          disabled={disabled}
        />
        <p className="text-xs text-muted-foreground">
          Press this hotkey after selecting text in any app to rephrase it
        </p>
      </div>
    </div>
  );
}
