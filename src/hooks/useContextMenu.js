import { useContext } from 'react';
import { ContextMenuContext } from '../providers/ContextMenuProvider';

/**
 * Hook for using the context menu functionality.
 * @returns {Object} Context menu functions and state.
 */
const useContextMenu = () => {
    const context = useContext(ContextMenuContext);

    if (!context) {
        throw new Error('useContextMenu must be used within a ContextMenuProvider');
    }

    return context;
};

export default useContextMenu;