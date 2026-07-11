# Cursor Statistics Review Prompt

Use this prompt when asking Cursor or another AI assistant to review a statistical method.

```text
Review this ClinSize statistical method for correctness and implementation risk.

Check:
1. Are the hypotheses clearly defined?
2. Are all inputs validated?
3. Is the formula or numerical algorithm documented?
4. Are sign conventions clear?
5. Are alpha and sidedness handled correctly?
6. Is rounding conservative?
7. Is achieved power recalculated after rounding?
8. Are warnings returned for important assumptions?
9. Are tests sufficient?
10. Are reference values traceable?

Return findings ordered by severity. Focus on statistical bugs, numerical risks, validation gaps, and unclear assumptions. Do not spend time on style unless it affects correctness or maintainability.
```

