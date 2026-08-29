import { render, screen } from '@testing-library/react';
import { expect, test } from 'vitest';
import { MoneyText } from './MoneyText';

test('renders formatted money with tabular class', () => {
  render(<MoneyText cents={123456} currency="EUR" />);
  expect(screen.getByText('€1,234.56')).toHaveClass('tabular-money');
});

test('defaults to neutral color for negatives', () => {
  render(<MoneyText cents={-100} currency="EUR" />);
  expect(screen.getByText('-€1.00')).toHaveClass('text-text-1');
});

test('colored mode colors by sign', () => {
  render(<MoneyText cents={-100} currency="EUR" colored />);
  expect(screen.getByText('-€1.00')).toHaveClass('text-status-extra-txt');
  render(<MoneyText cents={250000} currency="EUR" colored />);
  expect(screen.getByText('€2,500.00')).toHaveClass('text-status-matched-txt');
});
