import { useEffect, useId, useRef, useState, type ReactNode } from 'react';
import { CheckIcon, ChevronDownIcon } from 'lucide-react';
import { cn } from '@/shared/lib/utils';
import { Pill } from './StatusPill';

export interface ComboboxOption {
  value: string;
  label: string;
  badge?: string;
}
export interface ComboboxGroup {
  label: string;
  options: ComboboxOption[];
}

const fold = (s: string) => s.normalize('NFD').replace(/\p{M}/gu, '').toLowerCase();

/** Editable combobox (ARIA 1.2 list autocomplete): the input filters the grouped options by label;
 *  `footer(query)` options always follow, unfiltered (e.g. "Create …", "Cash"). Focus stays on the input. */
export function Combobox({
  label,
  value,
  displayValue,
  placeholder,
  groups,
  footer,
  onChange,
  description,
}: {
  label: string;
  value: string;
  /** Text shown while not searching (the current choice). */
  displayValue: string;
  placeholder?: string;
  groups: ComboboxGroup[];
  footer: (query: string) => ComboboxOption[];
  onChange: (value: string) => void;
  /** Rendered under the input and linked as its description. */
  description?: ReactNode;
}) {
  const id = useId();
  const inputRef = useRef<HTMLInputElement>(null);
  const [open, setOpen] = useState(false);
  // null = not searching: the input shows displayValue and the list is unfiltered.
  const [query, setQuery] = useState<string | null>(null);
  const [active, setActive] = useState(0);

  const q = fold((query ?? '').trim());
  const visible = groups
    .map((g) => ({ ...g, options: g.options.filter((o) => fold(o.label).includes(q)) }))
    .filter((g) => g.options.length > 0);
  const tail = footer((query ?? '').trim());
  const flat = [...visible.flatMap((g) => g.options), ...tail];
  const activeOption = open ? flat[Math.min(active, flat.length - 1)] : undefined;
  const optId = (i: number) => `${id}-opt-${i}`;

  function show() {
    if (open) return;
    // Start on the current account; on Cash or no choice start at the top so the groups stay in view.
    const current = visible.flatMap((g) => g.options).findIndex((o) => o.value === value);
    setActive(Math.max(0, current));
    setOpen(true);
  }
  function close() {
    setOpen(false);
    setQuery(null);
  }
  function choose(o: ComboboxOption) {
    onChange(o.value);
    close();
  }

  // Radix dialogs dismiss on Escape from a capture listener on document; a window capture listener
  // runs first, so an open list closes alone and the defaultPrevented event leaves the sheet open.
  useEffect(() => {
    if (!open) return;
    const onKey = (e: KeyboardEvent) => {
      if (e.key !== 'Escape' || document.activeElement !== inputRef.current) return;
      e.preventDefault();
      setOpen(false);
      setQuery(null);
    };
    window.addEventListener('keydown', onKey, true);
    return () => window.removeEventListener('keydown', onKey, true);
  }, [open]);

  const activeId = activeOption ? optId(flat.indexOf(activeOption)) : undefined;
  useEffect(() => {
    if (activeId) document.getElementById(activeId)?.scrollIntoView({ block: 'nearest' });
  }, [activeId]);

  function onKeyDown(e: React.KeyboardEvent<HTMLInputElement>) {
    if (e.key === 'ArrowDown' || e.key === 'ArrowUp') {
      e.preventDefault();
      if (!open) return show();
      if (e.altKey) {
        if (e.key === 'ArrowUp') close();
        return;
      }
      const step = e.key === 'ArrowDown' ? 1 : -1;
      setActive((a) => (Math.min(a, flat.length - 1) + step + flat.length) % flat.length);
    } else if (e.key === 'Enter' && activeOption) {
      e.preventDefault();
      choose(activeOption);
    }
  }

  let index = -1;
  const renderOption = (o: ComboboxOption) => {
    const i = ++index;
    const isActive = o === activeOption;
    return (
      <div
        key={o.value}
        id={optId(i)}
        role="option"
        aria-selected={isActive}
        onMouseMove={() => !isActive && setActive(i)}
        onClick={() => choose(o)}
        className={cn(
          'flex min-h-11 cursor-default items-center gap-2 rounded-sm px-2 py-1.5 text-sm transition-colors md:min-h-9',
          isActive && 'bg-accent-soft text-text-1',
        )}
      >
        <span className="min-w-0 flex-1 break-words">{o.label}</span>
        {o.badge && (
          <span className="shrink-0 whitespace-nowrap">
            <Pill>{o.badge}</Pill>
          </span>
        )}
        <CheckIcon
          aria-hidden="true"
          className={cn('size-4 shrink-0 text-accent', o.value !== value && 'invisible')}
        />
      </div>
    );
  };

  return (
    <div>
      <label htmlFor={`${id}-input`} className="mb-1 block text-sm text-text-2">
        {label}
      </label>
      <div className="relative">
        <input
          ref={inputRef}
          id={`${id}-input`}
          role="combobox"
          aria-expanded={open}
          aria-controls={`${id}-list`}
          aria-autocomplete="list"
          aria-activedescendant={activeId}
          aria-describedby={description ? `${id}-desc` : undefined}
          autoComplete="off"
          spellCheck={false}
          value={query ?? displayValue}
          placeholder={placeholder}
          onFocus={(e) => e.currentTarget.select()}
          // While not searching, a click selects the whole choice so typing replaces it; the default
          // mousedown would place the caret (Chrome collapses a selection even after onClick).
          onMouseDown={(e) => {
            if (query !== null) return;
            e.preventDefault();
            e.currentTarget.focus();
            e.currentTarget.select();
            show();
          }}
          onClick={show}
          onBlur={close}
          onChange={(e) => {
            setQuery(e.target.value);
            setActive(0);
            setOpen(true);
          }}
          onKeyDown={onKeyDown}
          className={cn(
            'flex h-9 w-full min-w-0 rounded-md border border-input bg-transparent py-1 pr-8 pl-3 text-base shadow-xs transition-[color,box-shadow] outline-none placeholder:text-muted-foreground md:text-sm',
            'focus-visible:border-ring focus-visible:ring-[3px] focus-visible:ring-ring/50',
          )}
        />
        <ChevronDownIcon
          aria-hidden="true"
          className="pointer-events-none absolute top-1/2 right-2.5 size-4 -translate-y-1/2 opacity-50"
        />
        {open && (
          <div
            id={`${id}-list`}
            role="listbox"
            aria-label={label}
            // Any press in the list (options, headings, padding, scrollbar) keeps focus in the input.
            onMouseDown={(e) => e.preventDefault()}
            className="absolute inset-x-0 top-full z-50 mt-1 max-h-72 overflow-y-auto overscroll-contain rounded-md border border-border-subtle bg-popover p-1 text-popover-foreground shadow-md"
          >
            {visible.map((g, gi) => (
              <div key={g.label} role="group" aria-labelledby={`${id}-g${gi}`}>
                <div
                  id={`${id}-g${gi}`}
                  role="presentation"
                  className="px-2 pt-2 pb-1 text-xs font-semibold text-text-3"
                >
                  {g.label}
                </div>
                {g.options.map(renderOption)}
              </div>
            ))}
            {tail.length > 0 && (
              <div
                role="presentation"
                className={cn(visible.length > 0 && 'mt-1 border-t border-border-subtle pt-1')}
              >
                {tail.map(renderOption)}
              </div>
            )}
          </div>
        )}
      </div>
      {description && (
        <p id={`${id}-desc`} role="status" className="mt-1 text-xs text-text-2">
          {description}
        </p>
      )}
    </div>
  );
}
