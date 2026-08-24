import { render, screen } from '@testing-library/react';

test('vitest runs', () => {
  render(<div>ok</div>);
  expect(screen.getByText('ok')).toBeInTheDocument();
});
