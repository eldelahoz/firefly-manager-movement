import { flushSync } from 'react-dom';

const MOVING = 'vt-move';

/** Animates the element tagged `data-vt-key={key}` from its old place to its new one (or out, if it is gone).
 *  `update` must change LOCAL React state: flushSync cannot flush React Query's deferred cache notifications.
 *  `after` runs once the DOM shows the new state (e.g. to restore focus). */
export function moveWithTransition(key: string, update: () => void, after?: () => void) {
  if (
    typeof document.startViewTransition !== 'function' ||
    window.matchMedia('(prefers-reduced-motion: reduce)').matches
  ) {
    flushSync(update);
    after?.();
    return;
  }
  const find = () => document.querySelector<HTMLElement>(`[data-vt-key="${key}"]`);
  let el = find();
  if (el) el.style.viewTransitionName = MOVING;
  const vt = document.startViewTransition(() => {
    flushSync(update);
    // Virtual rows are keyed by index, so the old element may now show another movement: untag it first.
    if (el) el.style.viewTransitionName = '';
    el = find();
    if (el) el.style.viewTransitionName = MOVING;
    after?.();
  });
  const untag = () => {
    if (el) el.style.viewTransitionName = '';
  };
  vt.finished.then(untag, untag);
}
