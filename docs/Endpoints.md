
# 

```typescript jsx
useEffect(() => {
  const fetchMetaData = async () => {
    try {
      const result = await invoke("get_meta_data");
      console.log("Fetched MetaData:", result);
    } catch (error) {
      console.error("Error fetching metadata:", error);
    }
  };

  fetchMetaData();
}, []);
```
