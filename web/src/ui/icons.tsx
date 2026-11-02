// The only icon source: Nucleo UI duotone files in /assets/icons (file name = role).
// Inlined as raw SVG so they take the text colour through currentColor.
import { memo, type CSSProperties } from "react";
import { cn } from "@/lib/utils";

const files = import.meta.glob("../../../assets/icons/*.svg", { query: "?raw", import: "default", eager: true }) as Record<string, string>;

export const ICONS: Record<string, string> = Object.fromEntries(
  Object.entries(files).map(([path, raw]) => [path.split("/").pop()!.replace(".svg", ""), raw]),
);

export type IconName = string;

/** Inner markup of an icon (paths only) for embedding in diagram SVG. */
export const iconInner = (name: string): string | null => {
  const raw = ICONS[name];
  if (!raw) return null;
  return raw.replace(/^<svg[^>]*>/, "").replace(/<\/svg>\s*$/, "");
};

interface Props { name: IconName; size?: number; className?: string; style?: CSSProperties; title?: string }

/** 16px in toolbar buttons, 14px in chips and rows. */
export const Icon = memo(function Icon({ name, size = 16, className, style, title }: Props) {
  const raw = ICONS[name] ?? "";
  return (
    <span
      role={title ? "img" : undefined}
      aria-label={title}
      aria-hidden={title ? undefined : true}
      className={cn("rv-icon inline-flex shrink-0", className)}
      style={{ width: size, height: size, ...style }}
      dangerouslySetInnerHTML={{ __html: raw }}
    />
  );
});

type P = { className?: string };
const make = (name: string, rotate = 0) => function Shim({ className }: P) {
  return <Icon name={name} size={14} className={className} style={rotate ? { transform: `rotate(${rotate}deg)` } : undefined} />;
};
// Drop-ins for the glyphs bundled with shadcn components.
export const ChevronRightIcon = make("chevron-right");
export const ChevronDownIcon = make("chevron-down");
export const ChevronUpIcon = make("chevron-down", 180);
export const CheckIcon = make("check");
export const XIcon = make("close");
