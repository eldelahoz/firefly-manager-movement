import { afterAll, afterEach, beforeAll, expect, test } from 'vitest';
import { setupServer } from 'msw/node';
import { http, HttpResponse } from 'msw';
import { api, ApiError } from './client';

const server = setupServer(
  http.get('/api/ok', () => HttpResponse.json({ value: 1 })),
  http.get('/api/fail', () => HttpResponse.json({ error: 'boom' }, { status: 502 })),
);
beforeAll(() => server.listen());
afterEach(() => server.resetHandlers());
afterAll(() => server.close());

test('api.get returns parsed json', async () => {
  expect(await api.get<{ value: number }>('/api/ok')).toEqual({ value: 1 });
});

test('api.get throws ApiError with status and message', async () => {
  const err = (await api.get('/api/fail').catch((e) => e)) as ApiError;
  expect(err).toBeInstanceOf(ApiError);
  expect(err.status).toBe(502);
  expect(err.message).toBe('boom');
});
