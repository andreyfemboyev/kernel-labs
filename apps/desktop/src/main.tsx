import { Channel, invoke } from "@tauri-apps/api/core";
import { useEffect, useRef, useState } from "react";
import { createRoot } from "react-dom/client";
import * as monaco from "monaco-editor/esm/vs/editor/editor.api";
import "monaco-editor/esm/vs/basic-languages/cpp/cpp.contribution";
import Worker from "monaco-editor/esm/vs/editor/editor.worker?worker";
import { inspect, workspaces, type Section } from "./bridge";
import "./style.css";

Object.assign(globalThis, { MonacoEnvironment: { getWorker: () => new Worker() } });

function Editor({ content, editable, theme, fontSize }: { content: string; editable: boolean; theme: string; fontSize: number }) {
  const host = useRef<HTMLDivElement>(null);
  const editor = useRef<monaco.editor.IStandaloneCodeEditor>(null);
  useEffect(() => {
    if (!host.current) return;
    const instance = monaco.editor.create(host.current, { automaticLayout: true, minimap: { enabled: false }, scrollBeyondLastLine: false });
    editor.current = instance;
    return () => {
      const model = instance.getModel();
      instance.dispose();
      model?.dispose();
      editor.current = null;
    };
  }, []);
  useEffect(() => {
    const instance = editor.current;
    if (!instance) return;
    instance.setValue(content);
    const model = instance.getModel();
    if (model) monaco.editor.setModelLanguage(model, editable ? "cpp" : "plaintext");
  }, [content, editable]);
  useEffect(() => {
    editor.current?.updateOptions({ readOnly: !editable, fontSize });
    monaco.editor.setTheme(theme === "light" ? "vs" : "vs-dark");
  }, [editable, theme, fontSize]);
  return <div className="editor" ref={host} />;
}

function App() {
  const [section, setSection] = useState<Section>(() => workspaces.find((item) => item.id === localStorage.getItem("section")) ?? workspaces[0]);
  const [root, setRoot] = useState(localStorage.getItem("root") ?? "");
  const [file, setFile] = useState("");
  const [profile, setProfile] = useState("");
  const [out, setOut] = useState("");
  const [plan, setPlan] = useState<unknown>(null);
  const [buildState, setBuildState] = useState<"idle" | "preview" | "running" | "cancelling">("idle");
  const [buildLog, setBuildLog] = useState<string[]>([]);
  const [secondary, setSecondary] = useState("");
  const [report, setReport] = useState("");
  const [state, setState] = useState<"idle" | "pending" | "ready" | "error">("idle");
  const [lang, setLang] = useState<"en" | "ru">(localStorage.getItem("lang") === "ru" ? "ru" : "en");
  const [theme, setTheme] = useState(localStorage.getItem("theme") ?? "dark");
  const [fontSize, setFontSize] = useState(Number(localStorage.getItem("font") ?? 13));
  const [palette, setPalette] = useState(false);
  const [navWidth, setNavWidth] = useState(300);
  const [filter, setFilter] = useState("");
  const active = useRef(0);
  const t = (en: string, ru: string) => lang === "en" ? en : ru;
  const path = section.input === "root" || section.input === "source" ? root : file;
  const message = {
    idle: t("Ready", "Готово"), pending: t("Reading…", "Чтение…"),
    ready: t("Complete", "Завершено"), error: t("Failed", "Ошибка"),
  }[state];

  function select(item: Section) {
    if (buildState !== "idle") return;
    setPlan(null);
    active.current++;
    setSection(item);
    localStorage.setItem("section", item.id);
    setReport("");
    setState("idle");
    setPalette(false);
    setFilter("");
  }

  useEffect(() => {
    const listener = (event: KeyboardEvent) => {
      if ((event.ctrlKey || event.metaKey) && event.shiftKey && event.key.toLowerCase() === "p") {
        event.preventDefault();
        setPalette((shown) => !shown);
      } else if (event.key === "Escape") setPalette(false);
    };
    window.addEventListener("keydown", listener);
    return () => window.removeEventListener("keydown", listener);
  }, []);
  useEffect(() => { document.documentElement.lang = lang; }, [lang]);

  async function run() {
    const request = ++active.current;
    setState("pending");
    setReport("");
    try {
      const data = await inspect(section, path, section.input === "source" ? file : secondary);
      if (request !== active.current) return;
      setReport(typeof data === "string" ? data : JSON.stringify(data, null, 2));
      setState("ready");
    } catch (error) {
      if (request !== active.current) return;
      setReport(String(error));
      setState("error");
    }
  }

  async function previewBuild() {
    const request = ++active.current;
    setBuildState("preview");
    setState("pending");
    setPlan(null);
    setReport("");
    setBuildLog([]);
    try {
      const reviewed = await invoke("preview_build", { request: { root, profile, out } });
      if (request !== active.current) return;
      setPlan(reviewed);
      setReport(JSON.stringify(reviewed, null, 2));
      setState("ready");
    } catch (error) {
      if (request !== active.current) return;
      setReport(String(error));
      setState("error");
    } finally {
      setBuildState("idle");
    }
  }

  async function startBuild() {
    if (!plan) return;
    const events = new Channel<{ stream: string; text: string }>();
    events.onmessage = ({ stream, text }) => setBuildLog((lines) => [...lines.slice(-299), `[${stream}] ${text.slice(0, 4096)}`]);
    setBuildState("running");
    setState("pending");
    try {
      const result = await invoke<{ build: { success: boolean } }>("run_build", { request: { root, profile, out }, reviewed: plan, events });
      setReport(JSON.stringify(result, null, 2));
      setState(result.build.success ? "ready" : "error");
    } catch (error) {
      setReport(String(error));
      setState("error");
    } finally {
      setPlan(null);
      setBuildState("idle");
    }
  }

  async function cancelBuild() {
    try {
      setBuildState("cancelling");
      await invoke("cancel_build");
    } catch (error) {
      setReport(String(error));
      setBuildState("running");
    }
  }

  const buildBusy = buildState !== "idle";
  return <div className="app" data-theme={theme}>
    <header><strong>KERNEL<span>//</span>LAB</strong><button onClick={() => setPalette(!palette)}>{t("Commands", "Команды")} <kbd>Ctrl+Shift+P</kbd></button></header>
    <div className="body" style={{ gridTemplateColumns: `53px ${navWidth}px minmax(0, 1fr)` }}>
      <nav aria-label={t("Activities", "Разделы")}>
        {workspaces.map((item) => <button key={item.id} title={item[lang]} aria-label={item[lang]} aria-current={item.id === section.id ? "page" : undefined} onClick={() => select(item)}>{item.short}</button>)}
      </nav>
      <aside>
        <h1>{section[lang]}</h1>
        {section.id === "settings" ? <div className="form">
          <label>{t("Language", "Язык")}<select value={lang} onChange={(e) => {
            const language = e.target.value === "ru" ? "ru" : "en";
            setLang(language); localStorage.setItem("lang", language);
          }}><option value="en">English</option><option value="ru">Русский</option></select></label>
          <label>{t("Theme", "Тема")}<select value={theme} onChange={(e) => {
            setTheme(e.target.value); localStorage.setItem("theme", e.target.value);
          }}><option value="dark">{t("Dark", "Тёмная")}</option><option value="light">{t("Light", "Светлая")}</option></select></label>
          <label>{t("Editor font size", "Размер шрифта")}<input type="range" min="10" max="24" value={fontSize} onChange={(e) => {
            setFontSize(Number(e.target.value)); localStorage.setItem("font", e.target.value);
          }} /></label>
          <label>{t("Sidebar width", "Ширина панели")}<input type="range" min="220" max="420" value={navWidth} onChange={(e) => setNavWidth(Number(e.target.value))} /></label>
        </div> : section.id === "build" ? <form className="form" onSubmit={(e) => { e.preventDefault(); void previewBuild(); }}>
          <label>{t("Workspace path", "Путь к проекту")}<input required disabled={buildBusy} value={root} onChange={(e) => {
            setRoot(e.target.value); setPlan(null); localStorage.setItem("root", e.target.value);
          }} /></label>
          <label>{t("JSON profile path", "Путь к профилю JSON")}<input required disabled={buildBusy} value={profile} onChange={(e) => { setProfile(e.target.value); setPlan(null); }} /></label>
          <label>{t("Output directory", "Каталог вывода")}<input required disabled={buildBusy} value={out} onChange={(e) => { setOut(e.target.value); setPlan(null); }} /></label>
          <button className="primary" disabled={buildBusy}>{t("Preview build", "План сборки")}</button>
          {plan != null && <button type="button" disabled={buildBusy} onClick={() => void startBuild()}>{t("Trust workspace and run", "Доверять проекту и запустить")}</button>}
          <button type="button" disabled={buildState !== "running"} onClick={() => void cancelBuild()}>{t("Cancel build", "Отменить сборку")}</button>
          <button type="button" disabled={buildBusy || state === "pending" || !root.trim()} onClick={() => { setPlan(null); setBuildLog([]); void run(); }}>{t("Build history", "История сборок")}</button>
        </form> : <form className="form" onSubmit={(e) => { e.preventDefault(); void run(); }}>
          {(section.input === "root" || section.input === "source") && <label>{t("Workspace path", "Путь к проекту")}<input required value={root} placeholder="/path/to/kernel" onChange={(e) => {
            setRoot(e.target.value); localStorage.setItem("root", e.target.value);
          }} /></label>}
          {(section.input === "file" || section.input === "pair" || section.input === "source") && <label>{
            section.input === "source" ? t("Relative source path", "Относительный путь") : section.input === "pair" ? t("Baseline path", "Путь к эталону") : t("File path", "Путь к файлу")
          }<input required value={file} onChange={(e) => setFile(e.target.value)} /></label>}
          {section.input === "pair" && <label>{t("Candidate path", "Путь к кандидату")}<input required value={secondary} onChange={(e) => setSecondary(e.target.value)} /></label>}
          <button className="primary" disabled={state === "pending"}>{t("Inspect", "Проверить")}</button>
        </form>}
      </aside>
      <main className={section.id === "build" && buildLog.length ? "has-log" : undefined}>
        <div className="tab">{section[lang]}<span>{section.id === "source" && state === "ready" ? t("Scratch buffer", "Черновик") : t("Report", "Отчёт")}</span></div>
        <Editor key={section.id} content={report} editable={section.id === "source" && state === "ready"} theme={theme} fontSize={fontSize} />
        {section.id === "build" && buildLog.length > 0 && <pre className="build-log" aria-label={t("Build output", "Вывод сборки")}>{buildLog.join("\n")}</pre>}
      </main>
    </div>
    <footer><span role="status">{message}</span><span>{root || t("No workspace selected", "Проект не выбран")}</span><span>v0.1.0-alpha</span></footer>
    {palette && <div className="overlay" onClick={() => setPalette(false)}><div className="palette" role="dialog" aria-modal="true" aria-label={t("Commands", "Команды")} onClick={(e) => e.stopPropagation()}>
      <input autoFocus aria-label={t("Filter commands", "Фильтр команд")} placeholder={t("Filter commands", "Фильтр команд")} value={filter} onChange={(e) => setFilter(e.target.value)} />
      {workspaces.filter((item) => item[lang].toLowerCase().includes(filter.toLowerCase())).map((item) => <button key={item.id} onClick={() => select(item)}>{item[lang]}</button>)}
    </div></div>}
  </div>;
}
createRoot(document.getElementById("root")!).render(<App />);
