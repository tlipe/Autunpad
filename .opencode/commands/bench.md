---
description: Run performance benchmarks and analyze bottlenecks
agent: rust-perf
---

Execute performance analysis and benchmarking:

1. Run `cargo bench` if benchmarks are configured
2. If criterion is set up, run criterion benchmarks and analyze results
3. Profile the code using available tools:
   - Check for allocation patterns in hot paths
   - Identify potential iterator misuse or unnecessary clones
   - Analyze memory layout and cache efficiency
4. If specific functions are mentioned in $ARGUMENTS, focus analysis on those
5. Provide concrete optimization recommendations with expected impact
6. Suggest benchmark code if none exists

Include before/after comparisons when possible.

Respond in Portuguese (Brazil) to the user.
