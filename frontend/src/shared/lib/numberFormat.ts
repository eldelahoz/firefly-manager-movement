import { useSyncExternalStore } from 'react';
import { isNumberFormat, type NumberFormat } from './money';

/** NFR-08 v0.1: client-only preference next to the language (`ffm.locale`), applied without Save. */
const KEY = 'ffm.numberFormat';
const listeners = new Set<() => void>();

// shortcut: with storage blocked the choice cannot be kept and stays auto; hold it in memory if that matters.
function read(): NumberFormat {
  try {
    const saved = localStorage.getItem(KEY);
    return isNumberFormat(saved) ? saved : 'auto';
  } catch {
    return 'auto'; // storage blocked (private mode): follow the language
  }
}

const notify = () => listeners.forEach((l) => l());
// One listener for the module: other tabs changing the preference re-render every amount.
window.addEventListener('storage', notify);

function subscribe(listener: () => void) {
  listeners.add(listener);
  return () => listeners.delete(listener);
}

export function setNumberFormat(format: NumberFormat) {
  try {
    localStorage.setItem(KEY, format);
  } catch {
    // storage blocked: nothing to persist; read() keeps returning auto
  }
  notify();
}

export function useNumberFormat(): NumberFormat {
  return useSyncExternalStore(subscribe, read);
}
