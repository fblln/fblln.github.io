#!/usr/bin/env python3
"""Report editorial review cues, not AI authorship or mandatory replacements.

Persist the earlier inline blog analysis so it can be rerun on any article.
Ignore code and references; retain captions, where repetitive prose also occurs.
"""
import argparse
import re
import statistics
from pathlib import Path

RULES = {
    'stock emphasis': r'\b(?:genuinely|crucially|importantly)\b',
    'editorial preamble': r'\b(?:it is worth (?:noting|stating|being)|the honest (?:answer|summary|question)|the (?:interesting|useful|important) (?:part|thing) is|the whole point|all of which raises)\b',
    'sweeping claim': r'\b(?:everyone else|everybody|anyone who has|nobody (?:does|remembers|reads))\b',
    'stock transition': r'\b(?:in conclusion|in summary|at the end of the day|delve|foster|leverage|game.changer)\b',
    'rhetorical contrast': r'\b(?:not (?:just|merely|only)\b[^.!?]{0,160}\bbut|it is not\b[^.!?]{0,160}\.\s*it is)\b',
}


def prose(text):
    """Mask non-prose with spaces, retaining offsets for useful line numbers.

    Whole figures must not be discarded: their captions are part of the article.
    Quoted source material is excluded rather than treated as the author's style.
    """
    def blank(match):
        return re.sub(r'[^\n]', ' ', match.group())

    for pattern in (
        r'\A\+\+\+\s*\n.*?^\+\+\+\s*$',
        r'^## References\b.*\Z',
        r'^\s*```[^\n]*\n.*?^\s*```[^\n]*$',
        r'^\s*~~~[^\n]*\n.*?^\s*~~~[^\n]*$',
        r'<svg\b.*?</svg>', r'<!--.*?-->', r'<[^>]+>',
        r'^>[^\n]*', r'`[^`\n]+`', r'"[^"\n]+"', r'“[^”]+”',
        r'\]\([^)]*\)', r'https?://\S+',
    ):
        text = re.sub(pattern, blank, text, flags=re.M | re.S)
    return text


def analyse(text):
    """Keep cues separate from repetition: a single use can be appropriate."""
    body = prose(text)
    words = re.findall(r"\b[\w’']+\b", body)
    sentences = [s.split() for s in re.split(r'[.!?]+', body) if s.strip()]
    findings = []
    for label, pattern in RULES.items():
        for match in re.finditer(pattern.replace(' ', r'\s+'), body, re.I):
            line = body.count('\n', 0, match.start()) + 1
            findings.append((line, label, ' '.join(match.group().split())))
    honest = len(re.findall(r'\bhonest(?:ly)?\b', body, re.I))
    return {
        'words': len(words),
        'mean_sentence_words': statistics.mean(map(len, sentences)) if sentences else 0,
        'honest': honest,
        'findings': sorted(findings),
    }


def main():
    """Default to published articles; explicit paths allow drafts and single posts."""
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('paths', nargs='*', type=Path)
    args = parser.parse_args()
    root = Path(__file__).resolve().parents[2]
    paths = args.paths or sorted((root / 'content/articles').glob('*.md'))
    for path in paths:
        result = analyse(path.read_text())
        print(f"{path}: {result['words']} words; approximate mean sentence length "
              f"{result['mean_sentence_words']:.1f}; honest/honestly: {result['honest']}")
        if result['honest'] > 1:
            print('  Repeated honest/honestly: consider keeping the strongest use and rephrasing the others.')
        for line, label, phrase in result['findings']:
            print(f'  {line}: {label}: {phrase}')
    print('Review cues only. Keep necessary technical contrasts and deliberate phrasing.')


if __name__ == '__main__':
    main()
