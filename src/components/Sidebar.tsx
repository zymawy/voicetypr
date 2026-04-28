import {
  SidebarContent,
  SidebarFooter,
  SidebarGroup,
  SidebarMenu,
  SidebarMenuButton,
  SidebarMenuItem,
  Sidebar as SidebarPrimitive,
} from "@/components/ui/sidebar";
import { cn } from "@/lib/utils";
import {
  Clock,
  Cpu,
  FileAudio,
  HelpCircle,
  Home,
  Info,
  Layers,
  RefreshCw,
  Settings2,
  Sparkles,
  Video,
} from "lucide-react";

interface SidebarProps {
  activeSection: string;
  onSectionChange: (section: string) => void;
}

const mainSections = [
  { id: "overview", label: "Overview", icon: Home },
  { id: "recordings", label: "History", icon: Clock },
  { id: "audio", label: "Upload", icon: FileAudio },
  { id: "general", label: "Settings", icon: Settings2 },
  { id: "models", label: "Models", icon: Cpu },
  { id: "formatting", label: "Formatting", icon: Sparkles },
  { id: "rephrase", label: "Rephrase", icon: RefreshCw },
  { id: "meetings", label: "Meetings", icon: Video },
  { id: "about", label: "About", icon: Info },
];

const bottomSections = [{ id: "advanced", label: "Advanced", icon: Layers }];

export function Sidebar({ activeSection, onSectionChange }: SidebarProps) {
  return (
    <SidebarPrimitive >
      <SidebarContent className="px-2">
        <SidebarGroup className="flex-1">
          <SidebarMenu>
            {mainSections.map((section) => {
              const Icon = section.icon;
              const isActive = activeSection === section.id;
              return (
                <SidebarMenuItem key={section.id}>
                  <SidebarMenuButton
                    onClick={() => onSectionChange(section.id)}
                    isActive={isActive}
                    className={cn(
                      "group relative rounded-lg px-3 py-2 hover:bg-accent/50 transition-colors",
                      isActive &&
                        "bg-accent text-accent-foreground font-medium",
                    )}
                  >
                    <Icon
                      className={cn(
                        "h-4 w-4 transition-transform group-hover:scale-110",
                        isActive && "text-primary",
                      )}
                    />
                    <span className="ml-2">{section.label}</span>
                    {isActive && (
                      <div className="absolute left-0 top-1/2 -translate-y-1/2 w-1 h-6 bg-primary rounded-r-full" />
                    )}
                  </SidebarMenuButton>
                </SidebarMenuItem>
              );
            })}
          </SidebarMenu>
        </SidebarGroup>

        <SidebarGroup>
          <SidebarMenu>
            {bottomSections.map((section) => {
              const Icon = section.icon;
              const isActive = activeSection === section.id;
              return (
                <SidebarMenuItem key={section.id}>
                  <SidebarMenuButton
                    onClick={() => onSectionChange(section.id)}
                    isActive={isActive}
                    className={cn(
                      "group relative rounded-lg px-3 py-2 hover:bg-accent/50 transition-colors",
                      isActive &&
                        "bg-accent text-accent-foreground font-medium",
                    )}
                  >
                    <Icon
                      className={cn(
                        "h-4 w-4 transition-transform group-hover:scale-110",
                        isActive && "text-primary",
                      )}
                    />
                    <span className="ml-2">{section.label}</span>
                    {isActive && (
                      <div className="absolute left-0 top-1/2 -translate-y-1/2 w-1 h-6 bg-primary rounded-r-full" />
                    )}
                  </SidebarMenuButton>
                </SidebarMenuItem>
              );
            })}

            <SidebarMenuItem>
              <SidebarMenuButton
                onClick={() => onSectionChange("help")}
                isActive={activeSection === "help"}
                className={cn(
                  "group relative rounded-lg px-3 py-2 hover:bg-accent/50 transition-colors",
                  activeSection === "help" &&
                    "bg-accent text-accent-foreground font-medium",
                )}
              >
                <HelpCircle
                  className={cn(
                    "h-4 w-4 transition-transform group-hover:scale-110",
                    activeSection === "help" && "text-primary",
                  )}
                />
                <span className="ml-2">Help</span>
                {activeSection === "help" && (
                  <div className="absolute left-0 top-1/2 -translate-y-1/2 w-1 h-6 bg-primary rounded-r-full" />
                )}
              </SidebarMenuButton>
            </SidebarMenuItem>
          </SidebarMenu>
        </SidebarGroup>
      </SidebarContent>
      <SidebarFooter className="border-t border-border/40 p-3" />
    </SidebarPrimitive>
  );
}
