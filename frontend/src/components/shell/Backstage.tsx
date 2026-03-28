import { brand } from "../../config/brand";

interface BackstageProps {
  onClose: () => void;
}

export function Backstage({ onClose }: BackstageProps) {
  return (
    <div className="fixed inset-0 z-50 flex">
      {/* Sidebar */}
      <div
        className="w-64 flex flex-col py-3"
        style={{ backgroundColor: "var(--brand-header-bg)", color: "var(--brand-header-text)" }}
      >
        <button
          className="flex items-center gap-2 px-4 py-2 text-sm hover:bg-white/10 transition-colors"
          onClick={onClose}
        >
          <span>&larr;</span>
          <span>Terug</span>
        </button>

        <div className="mt-4 flex flex-col">
          <button className="px-4 py-2.5 text-left text-sm hover:bg-white/10">
            Instellingen
          </button>
          <button className="px-4 py-2.5 text-left text-sm hover:bg-white/10">
            Info
          </button>
        </div>

        <div className="mt-auto px-4 py-3 opacity-50 text-xs">
          <div className="font-medium">
            {brand.namePrefix}
            <span style={{ color: "var(--brand-primary)" }}>{brand.nameAccent}</span>
          </div>
          <div>{brand.product} v0.1.0</div>
        </div>
      </div>

      {/* Panel */}
      <div className="flex-1 bg-blueprint-white p-8">
        <h2 className="text-xl font-semibold mb-4">Over {brand.product}</h2>
        <p className="text-text-muted text-sm">{brand.tagline}</p>
        <p className="text-text-subtle text-xs mt-4">
          Druk op <kbd className="px-1 py-0.5 bg-concrete rounded text-xs">Esc</kbd> om terug te gaan
        </p>
      </div>
    </div>
  );
}
