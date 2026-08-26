import type { UploadResponse } from '@/shared/api/dto';

export type MappingColumns = {
  date_column: string;
  amount_column: string;
  description_column: string;
};

// eslint-disable-next-line @typescript-eslint/no-unused-vars
export function MappingStep(_props: {
  probe: UploadResponse;
  onMapped: (c: MappingColumns) => void;
}) {
  return <div data-testid="step-mapping" />;
}
