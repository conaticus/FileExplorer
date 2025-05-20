Let's analyze the **time complexity** of your `fast_fuzzy_v2` trigram-based fuzzy search and compare it to other fuzzy matching algorithms.

---

## 1. **Empirical Scaling from Benchmark Data**

| Paths    | Avg Search Time (µs) |
|----------|----------------------|
| 10       | 8.05                 |
| 100      | 25.21                |
| 1,000    | 192.05               |
| 10,000   | 548.39               |
| 170,456  | 3,431.88             |

Let's look at the growth factor with 10× increases:
- 10 → 100:     ~3.1× slower
- 100 → 1,000:  ~7.6× slower
- 1,000 → 10,000: ~2.9× slower
- 10,000 → 170,456 (~17×): ~6.3× slower

This is **better than O(n)** (which would be 10× slower), and typically falls closer to **O(n^a)** where **a ≈ 0.5–0.7** for these data points. In other words, it is **sub-linear** scaling.

### Why is it so fast?
- The trigram index allows the search to quickly narrow down potential matches (most paths do not share rare trigrams).
- Only paths sharing trigrams with the query are considered for scoring.
- For sparse queries, this can be very close to O(1) for most searches.

---

## 2. **Theoretical Complexity of Your Trigram Algorithm**

### **Index Construction**
- **Build Index:** O(N * L), where N = number of paths, L = average path length (since you extract all trigrams from each path).

### **Query/Search**
- **Extract trigrams from query:** O(Q), Q = query length.
- **For each query trigram, lookup in index:** O(1) per trigram, assuming hash map.
- **Union of all path indices for matched trigrams:** Suppose on average, each trigram points to M << N paths.
- **Scoring and ranking:** O(R), R = number of candidate paths (usually << N).

So, **typical search complexity:**
> O(Q + S), where S = number of candidate paths for the trigrams in the query, and usually S << N.

This is typically **sub-linear** in N (i.e., O(N^a), a < 1), and often **amortized O(1)** for rare queries.

---

## 3. **Comparison to Other Fuzzy Matching Algorithms**

### **A. Levenshtein/Optimal String Alignment (OSA)**
- **Complexity:** O(N * M^2), where N = number of paths, M = average string length
- **Very slow for large path sets** (must compare every path)

### **B. Full Linear Scan (substring or regex)**
- **Complexity:** O(N * Q), where Q = query length
- **Slow** for large N

### **C. Trie/Prefix Tree**
- **Exact match:** O(Q)
- **Fuzzy match (with wildcards):** O(N), as every string may be checked for edit distance

### **D. Sublime/VSCode-style Fuzzy (e.g., FZF, Skim)**
- **Complexity:** O(N * Q), with heuristics for skipping
- **Very fast in practice for small N, but can become slow for large N**

### **E. Trigram/Bigram/NGram Index (your approach)**
- **Complexity:** O(Q + S), S << N in practice
- **Very fast** for large N if queries are specific
- **Scales sub-linearly** for practical queries

---

## 4. **Summary Table**

| Algorithm                      | Theoretical Complexity   | Practical Scaling | Suitability           |
|---------------------------------|-------------------------|-------------------|-----------------------|
| Levenshtein (brute force)      | O(N*M^2)                | Linear/Quadratic  | Slow for large N      |
| Substring/Regex (scan)         | O(N*Q)                  | Linear           | Slower for large N    |
| Trie/Prefix Tree (prefix)      | O(Q)                    | Sub-linear       | Fast for prefixes     |
| **Trigram Index (yours)**      | **O(Q+S)**              | **Sub-linear**   | **Best for large N**  |
| FZF/Sublime/Skim (fuzzy scan)  | O(N*Q)                  | Linear           | Fast for small N      |

---

## 5. **Conclusion**

- **Your fast fuzzy v2 trigram search is among the fastest approaches for large-scale fuzzy search.**
- It **scales sub-linearly** with the number of paths, which is far better than classic fuzzy matching.
- For large datasets, it massively outperforms Levenshtein, regex, and even FZF-style algorithms.

**Your observed scaling and real-world performance are excellent and among the best possible for fuzzy search at scale.**

If you want a plot of your empirical timing and a fit to O(N^a), let me know!