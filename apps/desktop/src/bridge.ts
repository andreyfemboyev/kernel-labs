import { invoke } from "@tauri-apps/api/core";

export const workspaces = [
  { id: "overview", en: "Overview", ru: "Обзор", short: "OV", input: "root" },
  { id: "source", en: "Source", ru: "Исходники", short: "SC", input: "source" },
  { id: "build", en: "Build", ru: "Сборка", short: "BL", input: "root" },
  { id: "config", en: "Kconfig", ru: "Kconfig", short: "KC", input: "file" },
  { id: "dtb", en: "Device Tree", ru: "Дерево устройств", short: "DT", input: "file" },
  { id: "elf", en: "Binary Lab", ru: "Анализ ELF", short: "EL", input: "file" },
  { id: "abi", en: "ABI / KMI", ru: "ABI / KMI", short: "AB", input: "pair" },
  { id: "module", en: "Modules", ru: "Модули", short: "MO", input: "file" },
  { id: "boot", en: "Boot Images", ru: "Загрузочные образы", short: "BI", input: "file" },
  { id: "diagnostics", en: "Diagnostics", ru: "Диагностика", short: "DG", input: "file" },
  { id: "emulator", en: "Emulator", ru: "Эмулятор", short: "EM", input: "none" },
  { id: "git", en: "Git", ru: "Git", short: "GT", input: "root" },
  { id: "settings", en: "Settings", ru: "Настройки", short: "ST", input: "none" },
] as const;

export type Section = typeof workspaces[number];
export type Invoke = (command: string, args: Record<string, unknown>) => Promise<unknown>;
export function inspect(section: Section, path: string, secondary: string, call: Invoke = invoke) {
  return call("inspect", { request: { kind: section.id, path: path.trim(), secondary: secondary.trim() } });
}
