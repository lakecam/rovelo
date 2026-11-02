// Mirrors the serde shapes of the Rust model (crates/rovelo-core/src/model.rs, brand.rs).
export type Id = string;
export type Rgb = [number, number, number];

export type Role = "primary" | "secondary" | "accent" | "neutral" | "success" | "warning" | "danger";
export const ROLES: Role[] = ["primary", "secondary", "accent", "neutral", "success", "warning", "danger"];

export type Shape =
  | "rect" | "rounded" | "pill" | "diamond" | "circle" | "parallelogram"
  | "hexagon" | "cylinder" | "document" | "screen" | "note";
export const SHAPES: Shape[] = [
  "rect", "rounded", "pill", "diamond", "circle", "parallelogram",
  "hexagon", "cylinder", "document", "screen", "note",
];

export type LineStyle = "solid" | "dashed" | "dotted";
export type Arrow = "end" | "both" | "none";
export type Direction = "down" | "right" | "left" | "up";
export type DiagramKind = "flowchart" | "user_flow" | "task_flow";
export type EdgeRouting = "straight" | "orthogonal" | "curved";
export type NodeFill = "solid" | "soft" | "outline";
export type Priority = "low" | "normal" | "high";

export interface Point { x: number; y: number }
export interface Size { w: number; h: number }
export interface Rect { x: number; y: number; w: number; h: number }

export interface Palette {
  primary: Rgb; secondary: Rgb; accent: Rgb; neutral: Rgb;
  success: Rgb; warning: Rgb; danger: Rgb;
}

export interface BrandKit {
  name: string;
  palette: Palette;
  background: Rgb;
  surface: Rgb;
  text: Rgb;
  muted_text: Rgb;
  font_family: string;
  heading_family: string;
  font_size: number;
  corner_radius: number;
  stroke_width: number;
  edge_routing: EdgeRouting;
  node_fill: NodeFill;
  /** Card (icon tile, the house look) or the classic outline shapes. */
  node_style?: "card" | "shape";
  shadow: boolean;
  show_grid: boolean;
  logo_svg: string | null;
}

export interface Node {
  id: Id; label: string; shape: Shape;
  pos: Point | null; size: Size | null;
  group: Id | null; role: Role; icon: string | null;
}
export interface Edge {
  id: Id; from: Id; to: Id; label: string;
  line: LineStyle; arrow: Arrow; role: Role | null;
}
export interface Group { id: Id; label: string; role: Role }
export interface Task {
  id: Id; title: string; done: boolean; node: Id | null;
  due: string | null; priority: Priority; tags: string[]; notes: string;
}

export interface Diagram {
  title: string; kind: DiagramKind; direction: Direction; brand: BrandKit;
  nodes: Node[]; edges: Edge[]; groups: Group[]; tasks: Task[];
}

export interface Diag { line: number; severity: "error" | "warning" | "info"; message: string }
export interface Parsed { diagram: Diagram; diagnostics: Diag[] }

export interface EdgeScene {
  id: Id; points: Point[]; label_at: Point; end_dir: Point; start_dir: Point;
  label_rect: Rect | null; svg: string; label_svg: string | null;
}
export interface Scene {
  defs: string; bounds: Rect;
  nodes: { id: Id; rect: Rect; svg: string }[];
  edges: EdgeScene[];
  groups: { id: Id; rect: Rect; svg: string }[];
  badges: { id: Id; svg: string }[];
}

export interface Selection { nodes: Id[]; edges: Id[]; groups: Id[] }
export const emptySelection = (): Selection => ({ nodes: [], edges: [], groups: [] });
