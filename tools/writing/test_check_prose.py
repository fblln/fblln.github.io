"""Regression cases for prose boundaries and useful editorial diagnostics."""
import unittest
from check_prose import analyse


class ProseTests(unittest.TestCase):
    def test_ignores_non_author_prose_but_retains_caption_and_line(self):
        source = '''+++
title = "honest"
+++
```python
honest = "honest"
```
<svg><text>honest</text></svg>
<figcaption>The honest answer.</figcaption>
> honest
"honest"
`honest`
## References
honest
'''
        result = analyse(source)
        self.assertEqual(result['honest'], 1)
        self.assertTrue(all(line == 8 for line, _, _ in result['findings']))

    def test_repetition_counts_whole_words_and_case(self):
        self.assertEqual(analyse('Honest. Honestly. Dishonest.')['honest'], 2)

    def test_multiline_preamble_and_contrast(self):
        labels = {label for _, label, _ in analyse(
            'The honest\nsummary. Everyone else does it. '
            'It is not a fix. It is a workaround.'
        )['findings']}
        self.assertTrue({'editorial preamble', 'sweeping claim',
                         'rhetorical contrast'} <= labels)

    def test_empty_and_technical_prose(self):
        self.assertEqual(analyse('')['mean_sentence_words'], 0)
        self.assertEqual(analyse('The counter identifies the applied generation.')['findings'], [])


if __name__ == '__main__':
    unittest.main()
