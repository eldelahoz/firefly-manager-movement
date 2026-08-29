import { useQuery } from '@tanstack/react-query';
import { Link } from 'react-router-dom';
import { useTranslation } from 'react-i18next';
import { getImports } from '@/shared/api/endpoints';
import { queryKeys } from '@/shared/lib/keys';
import { formatDate } from '@/shared/lib/dates';
import { StatusPill } from '@/shared/ui/StatusPill';
import { TableSkeleton } from '@/shared/ui/Skeleton';
import { Table, Td, Tr } from '@/shared/ui/Table';

export function RecentBatches() {
  const { t, i18n } = useTranslation();
  const { data, isPending } = useQuery({ queryKey: queryKeys.imports, queryFn: getImports });
  if (isPending) return <TableSkeleton rows={4} cols={3} />;
  const top = (data ?? []).slice(0, 8);
  return (
    <section className="min-w-0">
      <h2 className="mb-3 font-semibold">{t('dashboard.recentBatches')}</h2>
      <Table head={[t('imports.filename'), t('dashboard.date'), t('imports.status')]}>
        {top.map((b) => (
          <Tr key={b.id}>
            <Td className="w-2/5 max-w-[11rem] truncate font-medium sm:w-3/5 sm:max-w-[16rem]">
              <Link
                to={b.status !== 'parsed' ? `/results/${b.id}` : '/imports'}
                className="hover:text-accent"
              >
                {b.filename}
              </Link>
            </Td>
            <Td className="whitespace-nowrap text-text-2">
              {formatDate(b.created_at.slice(0, 10), i18n.language)}
            </Td>
            <Td>
              <StatusPill status={b.status} />
            </Td>
          </Tr>
        ))}
      </Table>
    </section>
  );
}
