import { useSyncExternalStore } from "react";
import { api } from "../lib/tauri";
import type { AppSettings, ProcessingState } from "../types";

export interface AppStoreSnapshot {
  settings: AppSettings | null;
  processingState: ProcessingState;
  processingError?: string;
}

let snapshot: AppStoreSnapshot = {
  settings: null,
  processingState: "Idle",
};
const listeners = new Set<() => void>();
let started = false;

const emit = () => {
  listeners.forEach((listener) => listener());
};

export const subscribeAppStore = (listener: () => void) => {
  listeners.add(listener);
  return () => listeners.delete(listener);
};

export const getAppStoreSnapshot = () => snapshot;

export const setAppSettings = (settings: AppSettings) => {
  snapshot = { ...snapshot, settings };
  emit();
};

export const setProcessingState = (
  processingState: ProcessingState,
  processingError?: string,
) => {
  snapshot = { ...snapshot, processingState, processingError };
  emit();
};

export const startAppStore = async () => {
  if (started) return () => undefined;
  started = true;

  try {
    setAppSettings(await api.getSettings());
  } catch (error) {
    console.error("Failed to initialize app settings:", error);
  }

  const unlisten = await api.onStateChange(({ state, error }) => {
    setProcessingState(state, error);
  });

  return () => {
    unlisten();
    started = false;
  };
};

export const useAppStore = () =>
  useSyncExternalStore(subscribeAppStore, getAppStoreSnapshot, getAppStoreSnapshot);
