// File open / save. Tauri dialogs inside the desktop shell; File System Access API in browsers that
// have it; a download link and a hidden file input everywhere else.
import type { FileRef } from "@/state/store";

export const DIAGRAM_EXT = ".rovelo.json";
export const BRAND_EXT = ".brand.json";

export const isTauri = () => typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;

export const slug = (s: string) =>
  s.trim().toLowerCase().replace(/[^a-z0-9]+/g, "-").replace(/^-+|-+$/g, "") || "untitled";

/** File name without the double extension. */
export const docName = (name: string) => name.replace(/\.(rovelo|brand)\.json$/i, "").replace(/\.(json|svg|png|txt|md|mmd)$/i, "");

interface Kind { desc: string; exts: string[]; mime: string }
export const KINDS = {
  diagram: { desc: "Rovelo diagram", exts: ["json"], mime: "application/json" },
  brand: { desc: "Brand kit", exts: ["json"], mime: "application/json" },
  svg: { desc: "SVG image", exts: ["svg"], mime: "image/svg+xml" },
  png: { desc: "PNG image", exts: ["png"], mime: "image/png" },
  text: { desc: "Text", exts: ["txt", "md", "mmd"], mime: "text/plain" },
} satisfies Record<string, Kind>;
export type KindName = keyof typeof KINDS;

const fsaTypes = (k: Kind) => [{ description: k.desc, accept: { [k.mime]: k.exts.map((e) => `.${e}`) } }];

type FsaWindow = Window & {
  showOpenFilePicker?: (o: unknown) => Promise<FileSystemFileHandle[]>;
  showSaveFilePicker?: (o: unknown) => Promise<FileSystemFileHandle>;
};
const fsa = () => window as FsaWindow;
const aborted = (e: unknown) => e instanceof DOMException && e.name === "AbortError";

export interface Picked { text: string; ref: FileRef }

/** Ask for a file and read it as text. Null when cancelled. */
export async function openText(kind: KindName): Promise<Picked | null> {
  const k: Kind = KINDS[kind];
  if (isTauri()) {
    const { open } = await import("@tauri-apps/plugin-dialog");
    const { readTextFile } = await import("@tauri-apps/plugin-fs");
    const path = await open({ multiple: false, filters: [{ name: k.desc, extensions: k.exts }] });
    if (!path || Array.isArray(path)) return null;
    return { text: await readTextFile(path), ref: { name: path.split(/[\\/]/).pop() ?? path, path } };
  }
  if (fsa().showOpenFilePicker) {
    try {
      const [handle] = await fsa().showOpenFilePicker!({ types: fsaTypes(k) });
      const file = await handle.getFile();
      return { text: await file.text(), ref: { name: file.name, handle } };
    } catch (e) {
      if (aborted(e)) return null;
      throw e;
    }
  }
  return new Promise((resolve) => {
    const input = document.createElement("input");
    input.type = "file";
    input.accept = k.exts.map((e) => `.${e}`).join(",");
    input.onchange = async () => {
      const f = input.files?.[0];
      resolve(f ? { text: await f.text(), ref: { name: f.name } } : null);
    };
    input.oncancel = () => resolve(null);
    input.click();
  });
}

export interface SaveOpts {
  kind: KindName;
  suggestedName: string; // with extension
  /** Overwrite this file without asking where. */
  ref?: FileRef | null;
}

/** Write text or bytes. Returns where it went, or null when cancelled. */
export async function saveData(data: string | Uint8Array, o: SaveOpts): Promise<FileRef | null> {
  const k: Kind = KINDS[o.kind];
  if (isTauri()) {
    const { save } = await import("@tauri-apps/plugin-dialog");
    const fs = await import("@tauri-apps/plugin-fs");
    const path = o.ref?.path ?? (await save({ defaultPath: o.suggestedName, filters: [{ name: k.desc, extensions: k.exts }] }));
    if (!path) return null;
    if (typeof data === "string") await fs.writeTextFile(path, data);
    else await fs.writeFile(path, data);
    return { name: path.split(/[\\/]/).pop() ?? path, path };
  }
  const blob = new Blob([data as BlobPart], { type: k.mime });
  if (fsa().showSaveFilePicker) {
    try {
      const handle = (o.ref?.handle as FileSystemFileHandle | undefined) ?? (await fsa().showSaveFilePicker!({ suggestedName: o.suggestedName, types: fsaTypes(k) }));
      const w = await handle.createWritable();
      await w.write(blob);
      await w.close();
      return { name: handle.name, handle };
    } catch (e) {
      if (aborted(e)) return null;
      throw e;
    }
  }
  const a = document.createElement("a");
  a.href = URL.createObjectURL(blob);
  a.download = o.suggestedName;
  a.click();
  setTimeout(() => URL.revokeObjectURL(a.href), 2000);
  return { name: o.suggestedName };
}

/** Rasterise an SVG string to PNG bytes at `scale`x. */
export async function svgToPng(svg: string, scale = 2): Promise<Uint8Array> {
  const m = /viewBox="[^"]*?([\d.]+) ([\d.]+)"/.exec(svg);
  const w = Number(/width="([\d.]+)"/.exec(svg)?.[1] ?? m?.[1] ?? 800);
  const h = Number(/height="([\d.]+)"/.exec(svg)?.[1] ?? m?.[2] ?? 600);
  const url = URL.createObjectURL(new Blob([svg], { type: "image/svg+xml" }));
  try {
    const img = new Image();
    img.decoding = "async";
    await new Promise<void>((res, rej) => {
      img.onload = () => res();
      img.onerror = () => rej(new Error("The SVG could not be rasterised"));
      img.src = url;
    });
    const c = document.createElement("canvas");
    c.width = Math.round(w * scale);
    c.height = Math.round(h * scale);
    const ctx = c.getContext("2d")!;
    ctx.scale(scale, scale);
    ctx.drawImage(img, 0, 0, w, h);
    const blob = await new Promise<Blob | null>((res) => c.toBlob(res, "image/png"));
    if (!blob) throw new Error("PNG export failed");
    return new Uint8Array(await blob.arrayBuffer());
  } finally {
    URL.revokeObjectURL(url);
  }
}
