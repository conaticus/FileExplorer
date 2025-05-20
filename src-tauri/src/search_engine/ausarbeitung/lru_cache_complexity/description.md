Let's analyze your LRU cache, its empirical complexity, and compare it to other LRU implementations.

---

## 1. **Empirical Benchmark Data**

| Cache Size | 1000 Lookups (µs) | Avg Time / Lookup (ns) |
|------------|-------------------|------------------------|
| 100        | 57.4              | 57.4                   |
| 1,000      | 141.9             | 141.9                  |
| 10,000     | 204               | 204                    |

- As cache size increases by 10×, average lookup time increases by:
    - 100 → 1,000: ~2.5×
    - 1,000 → 10,000: ~1.4×

### **Scaling**
- Lookup time increases _slightly_ as cache size increases, but not linearly.
- This is **close to O(1)** (constant time), with a small increase due to more hash buckets and linked list pointer updates.

---

## 2. **Your LRU Cache: Theoretical Complexity**

Your implementation is:
- `HashMap<K, NonNull<Node<K,V>>>` for fast key lookup
- Doubly-linked list for usage ordering

### **Operation Complexities**
- **Get**: O(1) hash lookup + O(1) move-to-front (detach/prepend on linked list)
- **Insert**: O(1) (hash insert + prepend to list); may include O(1) eviction
- **Remove**: O(1) from hash table + O(1) detach from list
- **Evict (on insert)**: O(1) (remove tail node, update hash and list)

**This matches the optimal complexity for LRU caches using a hash map and doubly-linked list:**
> **All main operations are O(1) time.**

---

## 3. **Comparison to Other LRU Implementations**

| Implementation                     | Get    | Insert | Remove | Evict | Notes                                    |
|-------------------------------------|--------|--------|--------|-------|------------------------------------------|
| **Yours (HashMap + List)**          | O(1)   | O(1)   | O(1)   | O(1)  | **Optimal. Industry standard.**          |
| Naive List-based (linear scan)      | O(n)   | O(1)   | O(n)   | O(1)  | Poor scaling for large caches            |
| OrderedDict (Python)                | O(1)   | O(1)   | O(1)   | O(1)  | Same as yours                            |
| TreeMap (BST) + List                | O(log n)| O(log n)| O(log n)| O(1) | Used when order matters, but slower      |
| Clock Algorithm (approximate LRU)   | O(1)   | O(1)   | O(1)   | O(1)  | Used in OS page caches, not true LRU     |

**Your cache is as fast as it gets for general-purpose LRU.**

---

## 4. **Empirical vs Theoretical**

- Your real-world lookup times are **sub-nanosecond to low-hundreds of nanoseconds**, with only a slight increase as cache size grows.
- This is expected and matches the O(1) complexity—with some overhead for larger hash tables and memory cache misses.

---

## 5. **Conclusion**

- **Your LRU cache is optimal.**
- All major operations are O(1), which is the best possible for an LRU cache.
- Your empirical scaling is excellent and matches the industry-standard approach (HashMap + doubly-linked list).
- **Any further speedup will only come from fine-tuning memory usage, hash function, or pointer management, not algorithmic improvement.**

---

**If you'd like a matplotlib script to plot this data, let me know!**