import { useLayoutEffect, useRef, useState } from 'react';
import { useVirtualizer } from '@tanstack/react-virtual';

export function VirtualList({
  count,
  renderItem,
  estimateSize = () => 56,
  header,
  role = 'presentation',
  itemRole = 'presentation',
}: {
  count: number;
  renderItem: (index: number) => React.ReactNode;
  estimateSize?: (index: number) => number;
  /** Rendered as a static strip above the scroll container so scrolling starts at the first row.
   *  Gets right padding equal to the scrollbar width so its columns stay aligned with the rows. */
  header?: React.ReactNode;
  role?: string;
  itemRole?: string;
}) {
  const parentRef = useRef<HTMLDivElement>(null);
  const [gutter, setGutter] = useState(0);
  const virtualizer = useVirtualizer({
    count,
    getScrollElement: () => parentRef.current,
    estimateSize,
  });
  useLayoutEffect(() => {
    const el = parentRef.current;
    if (!el) return;
    const measure = () => setGutter(el.offsetWidth - el.clientWidth);
    measure();
    const observer = new ResizeObserver(measure);
    observer.observe(el);
    return () => observer.disconnect();
  }, []);
  return (
    <>
      {header && (
        <div role="presentation" style={{ paddingRight: gutter }}>
          {header}
        </div>
      )}
      <div ref={parentRef} role={role} className="max-h-[60vh] overflow-y-auto">
        <div
          role="presentation"
          style={{ height: virtualizer.getTotalSize(), position: 'relative' }}
        >
          {virtualizer.getVirtualItems().map((v) => (
            <div
              key={v.key}
              role={itemRole}
              ref={virtualizer.measureElement}
              data-index={v.index}
              style={{
                position: 'absolute',
                top: 0,
                left: 0,
                width: '100%',
                transform: `translateY(${v.start}px)`,
              }}
            >
              {renderItem(v.index)}
            </div>
          ))}
        </div>
      </div>
    </>
  );
}
