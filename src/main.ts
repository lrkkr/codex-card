import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import "./styles.css";

type UsageSnapshot = {
  plan: string;
  session_used: number;
  weekly_used: number;
};

const refreshIntervalMs = 60_000;
const appWindow = getCurrentWindow();

const planName = getElement("plan-name");
const statusDot = getElement("status-dot");
const message = getElement("message");
const refreshButton = getButton("refresh-button");
const closeButton = getButton("close-button");

function getElement(id: string): HTMLElement {
  const element = document.getElementById(id);
  if (!element) {
    throw new Error(`Missing required element: ${id}`);
  }
  return element;
}

function getButton(id: string): HTMLButtonElement {
  const element = getElement(id);
  if (!(element instanceof HTMLButtonElement)) {
    throw new Error(`Element is not a button: ${id}`);
  }
  return element;
}

function updateMetric(name: "session" | "weekly", value: number): void {
  const percentage = Math.min(Math.max(Math.round(value), 0), 100);
  getElement(`${name}-text`).textContent = `${percentage}%`;
  getElement(`${name}-bar`).style.width = `${percentage}%`;

  const progressBar = getElement(`${name}-bar`).parentElement;
  progressBar?.setAttribute("aria-valuenow", String(percentage));
  progressBar?.classList.toggle("warning", percentage >= 80);
}

function setLoading(isLoading: boolean): void {
  refreshButton.disabled = isLoading;
  refreshButton.classList.toggle("spinning", isLoading);
}

async function updateUsage(): Promise<void> {
  setLoading(true);
  message.textContent = "Refreshing usage...";

  try {
    const snapshot = await invoke<UsageSnapshot>("fetch_openai_usage");
    planName.textContent = snapshot.plan;
    updateMetric("session", snapshot.session_used);
    updateMetric("weekly", snapshot.weekly_used);
    statusDot.classList.remove("error");
    message.textContent = `Updated ${new Date().toLocaleTimeString([], {
      hour: "2-digit",
      minute: "2-digit",
    })}`;
  } catch (error) {
    planName.textContent = "Usage unavailable";
    statusDot.classList.add("error");
    message.textContent = String(error);
  } finally {
    setLoading(false);
  }
}

refreshButton.addEventListener("click", () => void updateUsage());
closeButton.addEventListener("click", () => void appWindow.close());

void updateUsage();
window.setInterval(() => void updateUsage(), refreshIntervalMs);
