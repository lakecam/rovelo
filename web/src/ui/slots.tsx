// Swap points for React Bits. Each export is one visual flourish; today they use the FREE
// React Bits components (installed from https://reactbits.dev/r) and plain shadcn/Tailwind.
// To use a Pro component instead, change only the body of that slot. web/REACTBITS.md lists
// which Pro component is planned for which slot.
import type { CSSProperties, ReactNode } from "react";
import BlurText from "@/components/BlurText";
import CountUp from "@/components/CountUp";
import FadeContent from "@/components/FadeContent";
import SpotlightCard from "@/components/SpotlightCard";

/** Empty-state headline. Free: React Bits BlurText. */
export function SlotHeadline({ text }: { text: string }) {
  return (
    <BlurText
      text={text}
      className="justify-center text-2xl font-semibold text-ink"
      delay={60}
      animateBy="words"
    />
  );
}

/** Template card shell. Free: React Bits SpotlightCard with a faint accent light. */
export function SlotCard({ children, onClick, label }: { children: ReactNode; onClick: () => void; label: string }) {
  return (
    <SpotlightCard
      theme="light"
      spotlightColor="var(--color-zinc-400)"
      intensity={0.12}
      borderGlow={0.7}
      spotlightSize={180}
      className="h-40 w-[200px] !rounded-2xl !p-0"
      style={{
        "--spotlight-card-surface": "var(--rv-surface)",
        "--spotlight-card-border": "var(--rv-border)",
        "--spotlight-card-shadow": "var(--rv-shadow-hair)",
      } as CSSProperties}
    >
      <button type="button" onClick={onClick} aria-label={label} className="block size-full rounded-2xl p-2 text-left outline-offset-[-2px]">
        {children}
      </button>
    </SpotlightCard>
  );
}

/** Animated number (task progress). Free: React Bits CountUp. */
export function SlotNumber({ value }: { value: number }) {
  return <CountUp to={value} from={0} duration={0.6} startWhen trigger="load" className="tabular-nums" />;
}

/** Soft appearance of a floating panel: 200ms opacity, no slide. Free: React Bits FadeContent. */
export function SlotReveal({ children, className }: { children: ReactNode; className?: string }) {
  return (
    <FadeContent trigger="mount" duration={0.2} ease="power1.out" blur={false} initialOpacity={0} className={className}>
      {children}
    </FadeContent>
  );
}
