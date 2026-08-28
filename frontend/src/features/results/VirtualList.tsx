import { useRef } from 'react';
import { useVirtualizer } from '@tanstack/react-virtual';

export function VirtualList({
  count,
  renderItem,
  estimateSize = () => 56,
}: {
  count: number;
  renderItem: (index: number) => React.ReactNode;
  estimateSize?: (index: number) => number;
}) {
  const parentRef = useRef<HTMLDivElement>(null);
  const virtualizer = useVirtualizer({
    count,
    getScrollElement: () => parentRef.current,
    estimateSize,
  });
  return (
    <div ref={parentRef} className="max-h-[60vh] overflow-y-auto">
      <div style={{ height: virtualizer.getTotalSize(), position: 'relative' }}>
        {virtualizer.getVirtualItems().map((v) => (
          <div
            key={v.key}
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
  );
}
