// commitlint plugin: language + subject-shape rules (English-only policy).
// Deny-list keeps Spanish content words + high-signal function words with word
// boundaries; ambiguous short words (de/el/la/y) are excluded to avoid false
// positives like "de-Spanish" or "y-axis". Remaining Spanglish is caught by
// human review (see CONTRIBUTING.md - Git Workflow).
const SPANISH_CHARS = /[ñÑ¿¡áéíóúü]/;
const SPANISH_WORDS =
  /\b(apareamiento|integraci[oó]n|documentaci[oó]n|traducci[oó]n|pruebas?|restantes|monto|fecha|crea|corrige|elimina|actualiza|con|por|para|sin|un|una)\b/i;

export default {
  rules: {
    'subject-english': ({ subject }) => {
      if (!subject) return [true, ''];
      const problems = [];
      if (SPANISH_CHARS.test(subject)) problems.push('non-English characters');
      const m = subject.match(SPANISH_WORDS);
      if (m) problems.push(`Spanish word "${m[1]}"`);
      return [problems.length === 0, `subject must be in English: ${problems.join('; ')}`];
    },
    // commitlint passes (parsed, when, applicable) where applicable is the third
    // element of the rule config ([1,'always',72]) received as the scalar 72.
    // Warning band (not a hard cap): heads over 72 chars are readable in git log
    // but long for GitHub's one-line displays. Measures the whole header line,
    // like the X=100 hard cap (header-max-length).
    'header-soft-length': ({ header }, _when, target) => [
      !header || header.length <= target,
      `header is ${header?.length} chars (recommended <= ${target}; hard cap 100)`,
    ],
    'scope-single': ({ scope }) => [
      !scope || !/[,\s/]/.test(scope),
      `use one scope per commit, not "${scope}" (detail goes in the body)`,
    ],
  },
};
