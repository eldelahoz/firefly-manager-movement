import english from './scripts/commitlint/plugin-english.mjs';

export default {
  extends: ['@commitlint/config-conventional'],
  rules: {
    'type-enum': [2, 'always', ['feat', 'fix', 'docs', 'refactor', 'test', 'chore', 'ci']],
    'scope-enum': [
      2,
      'always',
      [
        'domain',
        'application',
        'infra',
        'presentation',
        'backend',
        'frontend',
        'matrix',
        'specs',
        'docs',
        'deps',
        'ci',
        'docker',
      ],
    ],
    'scope-empty': [2, 'never'],
    'scope-single': [2, 'always'],
    // subject-case disabled: its default (reject start/sentence-case) false-positives
    // on leading acronyms (UTC-noon, FF3, CSV, RF-02). Capitalization style
    // (lowercase imperative first word, proper nouns exempt) is documented and
    // review-enforced instead.
    'subject-case': [0],
    // X = 100 on the WHOLE header line (git convention). commitlint's
    // subject-max-length only measures the text after ':' - the built-in
    // header-max-length is the correct enforcement point.
    'header-max-length': [2, 'always', 100],
    'header-soft-length': [1, 'always', 72],
    // X = 120 on the WHOLE message (header + body + footers, trimmed), so the
    // header/body/footer per-line caps above can never add up to a long message.
    'message-total-max-length': [2, 'always', 120],
    'subject-english': [2, 'always'],
    'body-max-line-length': [2, 'always', 100],
    'footer-max-line-length': [2, 'always', 100],
  },
  plugins: [english],
};
