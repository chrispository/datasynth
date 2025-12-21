import {
  createCliRenderer,
  BoxRenderable,
  TextRenderable,
  InputRenderable,
  ScrollBoxRenderable,
  Renderable,
} from "@opentui/core";
import { spawn } from "child_process";
import path from "path";
import fs from "fs";

// --- Constants & Config ---
const COLORS = {
  bg: "#1e1e2e",           // Dark Blue-Grey
  panelBg: "#252535",      // Slightly lighter
  primary: "#89b4fa",      // Light Blue
  accent: "#f5c2e7",       // Pinkish
  success: "#a6e3a1",      // Green
  error: "#f38ba8",        // Red
  text: "#cdd6f4",         // White-ish
  textMuted: "#6c7086",    // Grey
  border: "#45475a",
  borderFocus: "#89b4fa",
};

const TOPICS_FILE = path.resolve(import.meta.dir, "../topics.txt");
const ENV_FILE = path.resolve(import.meta.dir, "../.env");
const PYTHON_SCRIPT = path.resolve(import.meta.dir, "../generate_emails.py");

// --- State Management ---
interface AppState {
  currentTab: "dashboard" | "settings" | "logs";
  topics: string[];
  selectedTopic: string | null;
  threads: string;
  totalDocs: string;
  geminiKey: string;
  useGemini: boolean;
  generatePdf: boolean;
  isRunning: boolean;
  logs: string;
  statusMessage: string;
}

const state: AppState = {
  currentTab: "dashboard",
  topics: [],
  selectedTopic: null,
  threads: "5",
  totalDocs: "25",
  geminiKey: "",
  useGemini: false,
  generatePdf: false,
  isRunning: false,
  logs: "Ready to generate...\n",
  statusMessage: "Ready",
};

// --- Helpers ---
function loadTopics() {
  try {
    const data = fs.readFileSync(TOPICS_FILE, "utf-8");
    state.topics = data.split("\n").filter((l) => l.trim() !== "");
  } catch (e) {
    state.topics = ["General Business", "Tech Support", "HR Inquiry"];
  }
}

function loadEnv() {
  if (fs.existsSync(ENV_FILE)) {
    const content = fs.readFileSync(ENV_FILE, "utf-8");
    const match = content.match(/GEMINI_API_KEY=(.*)/);
    if (match) state.geminiKey = match[1].trim();
  }
}

function saveEnv(apiKey: string) {
  let content = "";
  if (fs.existsSync(ENV_FILE)) {
    content = fs.readFileSync(ENV_FILE, "utf-8");
    if (content.includes("GEMINI_API_KEY=")) {
      content = content.replace(/GEMINI_API_KEY=.*/, `GEMINI_API_KEY=${apiKey}`);
    } else {
      content += `\nGEMINI_API_KEY=${apiKey}\n`;
    }
  } else {
    content = `GEMINI_API_KEY=${apiKey}\n`;
  }
  fs.writeFileSync(ENV_FILE, content);
}

// --- UI Components ---

// 1. Header
function createHeader(renderer: any) {
  const container = new BoxRenderable(renderer, {
    width: "100%",
    height: 3,
    borderStyle: "single",
    borderColor: COLORS.primary,
    backgroundColor: COLORS.panelBg,
    justifyContent: "space-between",
    alignItems: "center",
    paddingLeft: 2,
    paddingRight: 2,
  });

  const title = new TextRenderable(renderer, {
    content: " GSYN ",
    fg: COLORS.bg,
    bg: COLORS.primary,
  });
  
  const subtitle = new TextRenderable(renderer, {
    content: " Synthetic Data Generator ",
    fg: COLORS.text,
  });

  const titleBox = new BoxRenderable(renderer, {
    flexDirection: "row",
    alignItems: "center",
  });
  titleBox.add(title);
  titleBox.add(subtitle);
  container.add(titleBox);

  // Tabs
  const tabsBox = new BoxRenderable(renderer, {
    flexDirection: "row",
    gap: 2,
  });

  const tabs = ["Dashboard", "Settings", "Logs"];
  tabs.forEach((tab) => {
    const tName = tab.toLowerCase();
    const tText = new TextRenderable(renderer, {
      content: `[ ${tab} ]`,
      fg: state.currentTab === tName ? COLORS.primary : COLORS.textMuted,
    });
    
    // We can't easily make text clickable in this version without wrapping in box
    const tBox = new BoxRenderable(renderer, { height: 1 });
    tBox.add(tText);
    tBox.onMouseUp = () => {
      state.currentTab = tName as any;
      renderApp(renderer); // Re-render logic
    };
    tabsBox.add(tBox);
  });
  
  container.add(tabsBox);
  return container;
}

// 2. Footer
function createFooter(renderer: any) {
  const container = new BoxRenderable(renderer, {
    width: "100%",
    height: 1,
    backgroundColor: COLORS.primary,
    flexDirection: "row",
    justifyContent: "space-between",
    paddingLeft: 2,
    paddingRight: 2,
    marginTop: 0,
  });

  const status = new TextRenderable(renderer, {
    content: `STATUS: ${state.isRunning ? "RUNNING..." : state.statusMessage}`,
    fg: COLORS.bg,
  });

  const shortcuts = new TextRenderable(renderer, {
    content: "CTRL+C to Quit | Mouse to Navigate",
    fg: COLORS.bg,
  });

  container.add(status);
  container.add(shortcuts);
  return container;
}

// 3. Tab: Dashboard
function createDashboardTab(renderer: any) {
  const container = new BoxRenderable(renderer, {
    flexDirection: "row",
    width: "100%",
    flexGrow: 1,
    padding: 1,
    gap: 1,
  });

  // Left Column: Controls
  const leftCol = new BoxRenderable(renderer, {
    width: "40%",
    height: "100%",
    borderStyle: "single",
    borderColor: COLORS.border,
    title: " Configuration ",
    flexDirection: "column",
    padding: 1,
  });

  // Inputs
  const addInput = (label: string, value: string, onChange: (v: string) => void) => {
    const wrapper = new BoxRenderable(renderer, {
      width: "100%",
      height: 4,
      flexDirection: "column",
      marginBottom: 1,
    });
    const lbl = new TextRenderable(renderer, { content: label, fg: COLORS.text });
    wrapper.add(lbl);
    
    const inpBox = new BoxRenderable(renderer, {
      width: "100%",
      height: 3,
      borderStyle: "single",
      borderColor: COLORS.border,
      focusedBorderColor: COLORS.borderFocus,
    });
    const inp = new InputRenderable(renderer, {
      width: "100%",
      value: value,
      fg: COLORS.text,
      bg: COLORS.bg,
      cursorColor: COLORS.primary,
    });
    inpBox.add(inp);
    wrapper.add(inpBox);
    
    // Hacky binding
    inpBox.onMouseOut = () => onChange(inp.value); // Save on blur/out
    inpBox.onMouseDown = () => {}; // Focus handled by core?

    // Better binding: poll or rely on manual update before start?
    // We'll update state on start, but let's try to keep sync.
    inp.onInput = (v) => onChange(v); // Hypothetical event if supported
    // Fallback: we read from inputs when Start is clicked
    
    return { wrapper, inp }; // Return inp to read later
  };

  const threadsInput = addInput("Thread Count (Roots)", state.threads, (v) => state.threads = v);
  leftCol.add(threadsInput.wrapper);
  
  const docsInput = addInput("Total Documents", state.totalDocs, (v) => state.totalDocs = v);
  leftCol.add(docsInput.wrapper);

  // Selected Topic Display
  const topicDisplayBox = new BoxRenderable(renderer, {
     width: "100%",
     height: 3,
     borderStyle: "single",
     borderColor: COLORS.accent,
     title: " Active Topic ",
     marginTop: 1,
     justifyContent: "center",
     alignItems: "center",
  });
  const topicDisplayText = new TextRenderable(renderer, {
    content: state.selectedTopic || "Random (None Selected)",
    fg: COLORS.accent,
  });
  topicDisplayBox.add(topicDisplayText);
  leftCol.add(topicDisplayBox);

  // Start Button
  const startBtn = new BoxRenderable(renderer, {
    width: "100%",
    height: 3,
    backgroundColor: state.isRunning ? COLORS.error : COLORS.success,
    justifyContent: "center",
    alignItems: "center",
    marginTop: 2,
  });
  const startText = new TextRenderable(renderer, {
    content: state.isRunning ? "RUNNING (Wait...)" : "START GENERATION",
    fg: "#000000",
  });
  startBtn.add(startText);
  
  startBtn.onMouseUp = () => {
    // Sync inputs manually before start
    state.threads = threadsInput.inp.value;
    state.totalDocs = docsInput.inp.value;
    runGeneration(renderer);
  };
  
  leftCol.add(startBtn);
  container.add(leftCol);

  // Right Column: Topic Selection
  const rightCol = new BoxRenderable(renderer, {
    width: "60%",
    height: "100%",
    borderStyle: "single",
    borderColor: COLORS.border,
    title: " Select Topic ",
    padding: 0,
  });

  const scroll = new ScrollBoxRenderable(renderer, {
    width: "100%",
    height: "100%",
  });

  state.topics.forEach(topic => {
    const isSelected = state.selectedTopic === topic;
    const row = new BoxRenderable(renderer, {
      width: "100%",
      height: 1,
      paddingLeft: 1,
      backgroundColor: isSelected ? COLORS.panelBg : undefined,
    });
    const txt = new TextRenderable(renderer, {
      content: isSelected ? `> ${topic}` : `  ${topic}`,
      fg: isSelected ? COLORS.primary : COLORS.textMuted,
    });
    row.add(txt);
    
    row.onMouseUp = () => {
        state.selectedTopic = topic;
        topicDisplayText.content = topic;
        renderApp(renderer); // Refresh list styles
    };
    row.onMouseOver = () => {
        if (state.selectedTopic !== topic) {
             txt.fg = COLORS.text;
             row.backgroundColor = "#333333";
        }
    };
    row.onMouseOut = () => {
        if (state.selectedTopic !== topic) {
             txt.fg = COLORS.textMuted;
             row.backgroundColor = undefined;
        }
    };

    scroll.add(row);
  });

  rightCol.add(scroll);
  container.add(rightCol);

  return container;
}

// 4. Tab: Settings
function createSettingsTab(renderer: any) {
  const container = new BoxRenderable(renderer, {
    flexDirection: "column",
    width: "100%",
    flexGrow: 1,
    padding: 2,
    borderStyle: "single",
    borderColor: COLORS.border,
  });

  // API Key
  const apiBox = new BoxRenderable(renderer, {
      width: "100%",
      height: 5,
      marginBottom: 1,
  });
  const apiLabel = new TextRenderable(renderer, { content: "Gemini API Key:", fg: COLORS.text });
  apiBox.add(apiLabel);
  
  const apiInputWrapper = new BoxRenderable(renderer, {
      width: "100%",
      height: 3,
      borderStyle: "single",
      borderColor: COLORS.border,
  });
  const apiInput = new InputRenderable(renderer, {
      width: "100%",
      value: state.geminiKey,
      fg: COLORS.text,
      bg: COLORS.bg,
  });
  // Auto-save on change?
  apiInputWrapper.onMouseOut = () => {
      state.geminiKey = apiInput.value;
      saveEnv(state.geminiKey);
  };
  
  apiInputWrapper.add(apiInput);
  apiBox.add(apiInputWrapper);
  container.add(apiBox);

  // Toggles helper
  const addToggle = (label: string, value: boolean, onToggle: () => void) => {
      const row = new BoxRenderable(renderer, {
          width: "100%",
          height: 3,
          flexDirection: "row",
          alignItems: "center",
          marginBottom: 1,
      });
      
      const btn = new BoxRenderable(renderer, {
          width: 6,
          height: 3,
          borderStyle: "single",
          borderColor: value ? COLORS.success : COLORS.error,
          justifyContent: "center",
          alignItems: "center",
          marginRight: 2,
      });
      const btnTxt = new TextRenderable(renderer, {
          content: value ? "ON" : "OFF",
          fg: value ? COLORS.success : COLORS.error,
      });
      btn.add(btnTxt);
      btn.onMouseUp = onToggle;
      
      const lbl = new TextRenderable(renderer, { content: label, fg: COLORS.text });
      
      row.add(btn);
      row.add(lbl);
      return row;
  };

  container.add(addToggle("Enable Gemini LLM (Requires API Key)", state.useGemini, () => {
      state.useGemini = !state.useGemini;
      renderApp(renderer);
  }));

  container.add(addToggle("Generate PDF Files (Slower)", state.generatePdf, () => {
      state.generatePdf = !state.generatePdf;
      renderApp(renderer);
  }));

  return container;
}

// 5. Tab: Logs
function createLogsTab(renderer: any) {
    const container = new BoxRenderable(renderer, {
        width: "100%",
        flexGrow: 1,
        borderStyle: "single",
        borderColor: COLORS.border,
        title: " Execution Logs ",
    });

    const scroll = new ScrollBoxRenderable(renderer, {
        width: "100%",
        height: "100%",
        stickyScroll: true,
        stickyStart: "bottom",
    });

    const txt = new TextRenderable(renderer, {
        content: state.logs,
        fg: COLORS.textMuted,
        width: "100%",
    });
    
    // Auto-update log text ref?
    // We'll just re-render this component or update its content reference?
    // In OpenTUI, if we change 'content' prop, it should re-render on next tick? 
    // Actually, createLogsTab is called on renderApp. 
    // We need a way to keep this updated if we are NOT re-rendering the whole app constantly.
    // BUT, we are re-rendering on tab switch. If we are ON the logs tab, we need it to update.
    // We'll attach this text renderable to state so the runner can update it directly?
    // Or simpler: The runner updates state.logs. We need to trigger a repaint.
    
    // We'll assign it to a global ref so `runGeneration` can update it live.
    // Ideally we don't rebuild the whole UI every log line.
    
    activeLogText = txt; // Capture ref
    
    scroll.add(txt);
    container.add(scroll);
    return container;
}

let activeLogText: TextRenderable | null = null;
let rootContainer: BoxRenderable | null = null;

function renderApp(renderer: any) {
  if (!rootContainer) {
      rootContainer = new BoxRenderable(renderer, {
        flexDirection: "column",
        width: "100%",
        height: "100%",
        backgroundColor: COLORS.bg,
      });
      renderer.root.add(rootContainer);
  } else {
      // Clear children
      // renderer.root.remove(rootContainer); // No, we want to clear content of rootContainer
      // OpenTUI doesn't have clear(), create new root?
      // Hack: we just rebuild the children list if the lib supports it, 
      // or we just remove the old one and add a new one.
      renderer.root.children = []; 
      rootContainer = new BoxRenderable(renderer, {
        flexDirection: "column",
        width: "100%",
        height: "100%",
        backgroundColor: COLORS.bg,
      });
      renderer.root.add(rootContainer);
  }

  rootContainer.add(createHeader(renderer));

  // Content Area
  const contentBox = new BoxRenderable(renderer, {
      width: "100%",
      flexGrow: 1,
  });
  
  if (state.currentTab === "dashboard") {
      contentBox.add(createDashboardTab(renderer));
  } else if (state.currentTab === "settings") {
      contentBox.add(createSettingsTab(renderer));
  } else if (state.currentTab === "logs") {
      contentBox.add(createLogsTab(renderer));
  }
  
  rootContainer.add(contentBox);
  rootContainer.add(createFooter(renderer));
}

// --- Logic ---

function runGeneration(renderer: any) {
    if (state.isRunning) return;
    state.isRunning = true;
    state.statusMessage = "Generating...";
    state.logs += "\n--- STARTING GENERATION ---\n";
    state.currentTab = "logs"; // Switch to logs
    renderApp(renderer);

    const rootsVal = parseInt(state.threads) || 5;
    const totalDocsVal = parseInt(state.totalDocs) || 25;
    let steps = totalDocsVal - rootsVal;
    if (steps < 0) steps = 0;

    // Save key
    if (state.geminiKey) saveEnv(state.geminiKey);

    const args = ["--roots", rootsVal.toString(), "--steps", steps.toString()];
    if (state.selectedTopic) args.push("--topic", state.selectedTopic);
    if (state.generatePdf) args.push("--pdf");
    if (state.useGemini) args.push("--gemini");

    const child = spawn("python3", ["-u", PYTHON_SCRIPT, ...args], {
        cwd: path.resolve(import.meta.dir, ".."),
    });

    child.stdout.on("data", (data) => {
        const str = data.toString();
        state.logs += str;
        if (activeLogText) activeLogText.content = state.logs;
    });

    child.stderr.on("data", (data) => {
        const str = `[ERR] ${data.toString()}`;
        state.logs += str;
        if (activeLogText) activeLogText.content = state.logs;
    });

    child.on("close", (code) => {
        state.isRunning = false;
        state.statusMessage = code === 0 ? "Completed Successfully" : "Failed (See Logs)";
        state.logs += `\n--- FINISHED (Code ${code}) ---\n`;
        if (activeLogText) activeLogText.content = state.logs;
        // Optional: Re-render to update footer status
        renderApp(renderer);
    });
}

// --- Main ---

async function main() {
  loadTopics();
  loadEnv();

  const renderer = await createCliRenderer({
    exitOnCtrlC: true,
    useMouse: true,
  });

  renderApp(renderer);
  renderer.start();
}

main().catch(console.error);