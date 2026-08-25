import { render, screen } from '@testing-library/react';
import { expect, test } from 'vitest';
import { MoneyText } from './MoneyText';

test('renders formatted money with tabular class', () => {
  render(<MoneyText cents={123456} currency="EUR" />);
  expect(screen.getByText('€1,234.56')).toHaveClass('tabular-money');
});

test('negative renders red class', () => {
  render(<MoneyText cents={-100} currency="EUR" />);
  expect(screen.getByText('-€1.00')).toHaveClass('text-status-extra-txt');
});
