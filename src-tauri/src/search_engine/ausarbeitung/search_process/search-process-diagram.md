graph TD
User[User enters search query] --> UI[UI sends query]
UI --> Engine[AutocompleteEngine]

    subgraph "Search Process"
        Engine --> Normalize[Normalize query]
        Normalize --> LRU[LRU Cache]
        
        LRU -- "Cache Hit" --> Validate[Validate path exists]
        Validate -- "Path exists" --> CachedResults[Return cached result]
        CachedResults -- "Return" --> Results
        Validate -- "Path doesn't exist" --> RemoveCache[Remove from cache]
        RemoveCache --> Radix
        
        LRU -- "Cache Miss" --> Radix[Adaptive Radix Trie]
        Radix --> EnoughCheck{Enough results?}
        
        EnoughCheck -- "Yes" --> Ranking
        EnoughCheck -- "No" --> Fuzzy[Fuzzy Search]
        Fuzzy --> Ranking
        
        subgraph "Context Factors"
            CD[Current Directory] --> Ranking
            FR[Frequency] --> Ranking
            RR[Recency] --> Ranking
            EW[Extensions] --> Ranking
            EF[Exact File Matches] --> Ranking
        end
        
        Ranking[Context-Aware Ranker] --> CacheTop[Cache top result]
        CacheTop --> RecordUsage[Record usage of top result]
        RecordUsage --> LimitResults[Limit to max results]
    end
    
    LimitResults --> Results[Return results]
    Results --> UIDisplay[UI displays results]
    
    subgraph "Background Process"
        FSW[File System Watcher] -- "File Changes" --> UQ[Update Queue]
        UQ --> Engine
        BII[Background Indexer] --> Engine
    end
    
    classDef primary fill:#6495ED,stroke:#333,stroke-width:2px,color:white;
    classDef secondary fill:#90EE90,stroke:#333,stroke-width:1px;
    classDef tertiary fill:#FFB6C1,stroke:#333,stroke-width:1px;
    classDef result fill:#FFA500,stroke:#333,stroke-width:2px;
    
    class Engine,Normalize primary;
    class Radix,Fuzzy,LRU,Ranking secondary;
    class CD,FR,RR,EW,BII,FSW,EF tertiary;
    class Results,UIDisplay result;
    style EF color:#000000
