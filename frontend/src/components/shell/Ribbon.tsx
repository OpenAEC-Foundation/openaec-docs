import { useState } from "react";

interface RibbonProps {
  onOpenBackstage: () => void;
}

export function Ribbon({ onOpenBackstage }: RibbonProps) {
  const [activeTab, setActiveTab] = useState("start");

  return (
    <div className="border-b border-border bg-white">
      {/* Tab bar */}
      <div className="flex items-center h-8 px-2 gap-0.5 border-b border-border/50">
        <button
          className="px-3 h-7 text-xs font-semibold rounded-sm transition-colors"
          style={{ color: "var(--brand-header-text)", backgroundColor: "var(--brand-header-bg)" }}
          onClick={onOpenBackstage}
        >
          Bestand
        </button>
        {["start", "documenten", "beeld"].map((tab) => (
          <button
            key={tab}
            className={`px-3 h-7 text-xs font-medium rounded-sm transition-colors capitalize ${
              activeTab === tab
                ? "bg-amber/10 text-amber"
                : "text-text-muted hover:bg-concrete"
            }`}
            onClick={() => setActiveTab(tab)}
          >
            {tab}
          </button>
        ))}
      </div>

      {/* Tab content */}
      <div className="h-[72px] flex items-center px-3 gap-6">
        {activeTab === "start" && (
          <>
            <RibbonGroup label="Project">
              <RibbonButton icon="+" label="Nieuw" />
              <RibbonButton icon="&#x21BB;" label="Vernieuwen" />
            </RibbonGroup>
            <RibbonGroup label="Bestanden">
              <RibbonButton icon="&#x2191;" label="Uploaden" />
              <RibbonButton icon="&#x1F4C1;" label="Nieuwe map" />
            </RibbonGroup>
            <RibbonGroup label="Zoeken">
              <RibbonButton icon="&#x1F50D;" label="Zoeken" />
            </RibbonGroup>
          </>
        )}
        {activeTab === "documenten" && (
          <>
            <RibbonGroup label="Status">
              <RibbonButton icon="&#x25CB;" label="Concept" />
              <RibbonButton icon="&#x25CE;" label="Review" />
              <RibbonButton icon="&#x2713;" label="Goedgekeurd" />
            </RibbonGroup>
            <RibbonGroup label="Weergave">
              <RibbonButton icon="&#x2630;" label="Lijst" />
              <RibbonButton icon="&#x2637;" label="Grid" />
            </RibbonGroup>
          </>
        )}
        {activeTab === "beeld" && (
          <RibbonGroup label="Panelen">
            <RibbonButton icon="&#x25A1;" label="Mappenstructuur" />
            <RibbonButton icon="&#x25A3;" label="Details" />
          </RibbonGroup>
        )}
      </div>
    </div>
  );
}

function RibbonGroup({ label, children }: { label: string; children: React.ReactNode }) {
  return (
    <div className="flex flex-col items-center">
      <div className="flex items-center gap-1 h-[52px]">{children}</div>
      <span className="text-[10px] text-text-subtle mt-0.5">{label}</span>
    </div>
  );
}

function RibbonButton({ icon, label }: { icon: string; label: string }) {
  return (
    <button
      className="flex flex-col items-center justify-center w-14 h-[48px] rounded-sm
                 text-text-muted hover:bg-concrete hover:text-text transition-colors"
      title={label}
    >
      <span className="text-lg leading-none">{icon}</span>
      <span className="text-[10px] mt-1 leading-none">{label}</span>
    </button>
  );
}
