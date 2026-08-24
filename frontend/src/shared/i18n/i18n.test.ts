import en from './locales/en/translation.json';
import fr from './locales/fr/translation.json';
import es from './locales/es/translation.json';

function keyPaths(obj: Record<string, unknown>, prefix = ''): string[] {
  return Object.entries(obj).flatMap(([k, v]) =>
    typeof v === 'object' && v !== null
      ? keyPaths(v as Record<string, unknown>, `${prefix}${k}.`)
      : [`${prefix}${k}`],
  );
}

test('locales en/fr/es have identical key sets', () => {
  const enKeys = keyPaths(en).sort();
  expect(keyPaths(fr).sort()).toEqual(enKeys);
  expect(keyPaths(es).sort()).toEqual(enKeys);
});
