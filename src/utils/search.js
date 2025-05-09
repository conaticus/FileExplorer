import { invoke } from '@tauri-apps/api';

/**
 * Search options for configuring search behavior.
 * @typedef {Object} SearchOptions
 * @property {boolean} caseSensitive - Whether the search should be case-sensitive.
 * @property {boolean} matchWholeWord - Whether to match whole words only.
 * @property {boolean} useRegex - Whether to interpret the query as a regular expression.
 * @property {boolean} includeHidden - Whether to include hidden files in the search.
 * @property {Array<string>} fileTypes - File types to include in the search.
 * @property {string} searchIn - Directory to search in.
 */

/**
 * Default search options.
 * @type {SearchOptions}
 */
export const DEFAULT_SEARCH_OPTIONS = {
    caseSensitive: false,
    matchWholeWord: false,
    useRegex: false,
    includeHidden: false,
    fileTypes: [],
    searchIn: '/',
};

/**
 * Search for files and directories based on a query string.
 * @param {string} query - The search query.
 * @param {SearchOptions} options - Search options.
 * @returns {Promise<Object>} - Search results with files and directories.
 */
export const searchFiles = async (query, options = DEFAULT_SEARCH_OPTIONS) => {
    if (!query || query.trim() === '') {
        return { files: [], directories: [] };
    }

    try {
        // This is a placeholder since we don't have a direct search API in the docs
        // In a real implementation, this would call a Tauri backend function
        console.log('Searching for:', query, 'with options:', options);

        // For now, return mock data based on the query
        return mockSearchResults(query, options);
    } catch (error) {
        console.error('Search error:', error);
        throw new Error(`Search failed: ${error.message}`);
    }
};

/**
 * Generate mock search results for testing.
 * @param {string} query - The search query.
 * @param {SearchOptions} options - Search options.
 * @returns {Object} - Mock search results.
 */
const mockSearchResults = (query, options) => {
    const { caseSensitive, matchWholeWord, searchIn } = options;

    // Normalize query for comparison based on options
    const normalizedQuery = caseSensitive ? query : query.toLowerCase();

    // Mock file and directory data
    const mockData = [
        {
            name: 'Document.pdf',
            path: `${searchIn}/Document.pdf`,
            is_symlink: false,
            access_rights_as_string: 'rw-r--r--',
            access_rights_as_number: 33188,
            size_in_bytes: 1503745,
            created: '2023-05-15 10:30:00',
            last_modified: '2023-05-15 10:30:00',
            accessed: '2023-05-15 10:30:00',
            isDirectory: false,
        },
        {
            name: 'Readme.txt',
            path: `${searchIn}/Readme.txt`,
            is_symlink: false,
            access_rights_as_string: 'rw-r--r--',
            access_rights_as_number: 33188,
            size_in_bytes: 2345,
            created: '2023-04-10 08:15:00',
            last_modified: '2023-04-10 08:15:00',
            accessed: '2023-04-10 08:15:00',
            isDirectory: false,
        },
        {
            name: 'Projects',
            path: `${searchIn}/Projects`,
            is_symlink: false,
            access_rights_as_string: 'rwxr-xr-x',
            access_rights_as_number: 16877,
            size_in_bytes: 4096,
            sub_file_count: 12,
            sub_dir_count: 3,
            created: '2023-03-20 14:45:00',
            last_modified: '2023-03-20 14:45:00',
            accessed: '2023-03-20 14:45:00',
            isDirectory: true,
        },
        {
            name: 'image.png',
            path: `${searchIn}/image.png`,
            is_symlink: false,
            access_rights_as_string: 'rw-r--r--',
            access_rights_as_number: 33188,
            size_in_bytes: 245760,
            created: '2023-05-02 11:20:00',
            last_modified: '2023-05-02 11:20:00',
            accessed: '2023-05-02 11:20:00',
            isDirectory: false,
        },
        {
            name: 'Documents',
            path: `${searchIn}/Documents`,
            is_symlink: false,
            access_rights_as_string: 'rwxr-xr-x',
            access_rights_as_number: 16877,
            size_in_bytes: 4096,
            sub_file_count: 8,
            sub_dir_count: 2,
            created: '2023-02-15 09:30:00',
            last_modified: '2023-02-15 09:30:00',
            accessed: '2023-02-15 09:30:00',
            isDirectory: true,
        },
        {
            name: 'project_documentation.md',
            path: `${searchIn}/project_documentation.md`,
            is_symlink: false,
            access_rights_as_string: 'rw-r--r--',
            access_rights_as_number: 33188,
            size_in_bytes: 8976,
            created: '2023-04-25 16:40:00',
            last_modified: '2023-04-25 16:40:00',
            accessed: '2023-04-25 16:40:00',
            isDirectory: false,
        },
        {
            name: 'todo.txt',
            path: `${searchIn}/todo.txt`,
            is_symlink: false,
            access_rights_as_string: 'rw-r--r--',
            access_rights_as_number: 33188,
            size_in_bytes: 567,
            created: '2023-05-08 13:10:00',
            last_modified: '2023-05-08 13:10:00',
            accessed: '2023-05-08 13:10:00',
            isDirectory: false,
        },
    ];

    // Filter based on the search query
    const filteredData = mockData.filter(item => {
        const itemName = caseSensitive ? item.name : item.name.toLowerCase();

        if (matchWholeWord) {
            // Split the name into words and check if any whole word matches
            const words = itemName.split(/\W+/);
            return words.some(word => word === normalizedQuery);
        } else {
            // Check if the name contains the query
            return itemName.includes(normalizedQuery);
        }
    });

    // Separate files and directories
    const files = filteredData.filter(item => !item.isDirectory);
    const directories = filteredData.filter(item => item.isDirectory);

    return { files, directories };
};

/**
 * Parse a search query to extract special tokens.
 * @param {string} query - The raw search query.
 * @returns {Object} - Parsed query and extracted options.
 */
export const parseSearchQuery = (query) => {
    const options = { ...DEFAULT_SEARCH_OPTIONS };
    let cleanQuery = query.trim();

    // Extract file type filter
    const typeMatch = cleanQuery.match(/type:(\w+)/i);
    if (typeMatch) {
        options.fileTypes = [typeMatch[1].toLowerCase()];
        cleanQuery = cleanQuery.replace(typeMatch[0], '').trim();
    }

    // Extract case sensitivity
    if (cleanQuery.includes('case:sensitive')) {
        options.caseSensitive = true;
        cleanQuery = cleanQuery.replace('case:sensitive', '').trim();
    } else if (cleanQuery.includes('case:insensitive')) {
        options.caseSensitive = false;
        cleanQuery = cleanQuery.replace('case:insensitive', '').trim();
    }

    // Extract whole word matching
    if (cleanQuery.includes('word:whole')) {
        options.matchWholeWord = true;
        cleanQuery = cleanQuery.replace('word:whole', '').trim();
    }

    // Extract regex flag
    if (cleanQuery.includes('regex:true')) {
        options.useRegex = true;
        cleanQuery = cleanQuery.replace('regex:true', '').trim();
    }

    // Extract hidden files inclusion
    if (cleanQuery.includes('hidden:true')) {
        options.includeHidden = true;
        cleanQuery = cleanQuery.replace('hidden:true', '').trim();
    }

    return {
        query: cleanQuery,
        options,
    };
};

/**
 * Format a search query with given options.
 * @param {string} query - The raw search query.
 * @param {SearchOptions} options - Search options.
 * @returns {string} - Formatted search query with option tokens.
 */
export const formatSearchQuery = (query, options = DEFAULT_SEARCH_OPTIONS) => {
    const {
        caseSensitive,
        matchWholeWord,
        useRegex,
        includeHidden,
        fileTypes,
    } = options;

    let formattedQuery = query.trim();

    // Add options as tokens
    if (caseSensitive) {
        formattedQuery += ' case:sensitive';
    }

    if (matchWholeWord) {
        formattedQuery += ' word:whole';
    }

    if (useRegex) {
        formattedQuery += ' regex:true';
    }

    if (includeHidden) {
        formattedQuery += ' hidden:true';
    }

    if (fileTypes.length > 0) {
        formattedQuery += ` type:${fileTypes[0]}`;
    }

    return formattedQuery;
};