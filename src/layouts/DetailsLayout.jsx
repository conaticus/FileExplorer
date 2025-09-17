import React, { useState, useRef, useEffect } from 'react';
import { useFileSystem } from '../providers/FileSystemProvider';
import DetailsPanel from '../components/explorer/DetailsPanel';
import './detailsLayout.css';

/**
 * DetailsLayout component that provides a resizable details panel.
 * This component renders a main content area alongside a resizable details panel.
 *
 * @param {Object} props - Component props
 * @param {React.ReactNode} props.children - Child components to render in the main content area
 * @returns {JSX.Element} The DetailsLayout component
 */
const DetailsLayout = ({ children }) => {
    const { selectedItems } = useFileSystem();
    const [panelWidth, setPanelWidth] = useState(300); // Default width
    const [isResizing, setIsResizing] = useState(false);
    const containerRef = useRef(null);
    const startXRef = useRef(0);
    const startWidthRef = useRef(0);

    /**
     * Handles the start of a resize operation.
     * Sets up necessary state and styles for resizing.
     *
     * @param {React.MouseEvent} e - The mouse down event
     */
    const handleResizeStart = (e) => {
        setIsResizing(true);
        startXRef.current = e.clientX;
        startWidthRef.current = panelWidth;

        // Prevent text selection during resize
        document.body.style.userSelect = 'none';
        document.body.style.cursor = 'col-resize';
    };

    /**
     * Effect hook to handle resizing operations.
     * Adds event listeners for mouse movements and cleanup.
     */
    useEffect(() => {
        /**
         * Handles resize calculations during mouse movement.
         *
         * @param {MouseEvent} e - The mouse move event
         */
        const handleResize = (e) => {
            if (!isResizing) return;

            const containerWidth = containerRef.current?.offsetWidth || 0;
            const deltaX = startXRef.current - e.clientX;
            const newWidth = Math.min(
                Math.max(200, startWidthRef.current + deltaX), // Min 200px, max based on delta
                containerWidth - 400 // Ensure main content has at least 400px
            );

            setPanelWidth(newWidth);
        };

        /**
         * Handles the end of a resize operation.
         * Resets styles and state.
         */
        const handleResizeEnd = () => {
            setIsResizing(false);
            document.body.style.userSelect = '';
            document.body.style.cursor = '';
        };

        if (isResizing) {
            window.addEventListener('mousemove', handleResize);
            window.addEventListener('mouseup', handleResizeEnd);
        }

        return () => {
            window.removeEventListener('mousemove', handleResize);
            window.removeEventListener('mouseup', handleResizeEnd);
        };
    }, [isResizing]);

    return (
        <div ref={containerRef} className="details-layout-container">
            {/* Main content */}
            <div
                className="details-layout-main"
                style={{
                    width: `calc(100% - ${panelWidth}px)`,
                }}
            >
                {children}
            </div>

            {/* Resize handle */}
            <div
                className="details-layout-resize-handle"
                onMouseDown={handleResizeStart}
            ></div>

            {/* Details panel */}
            <div
                className="details-layout-panel"
                style={{ width: `${panelWidth}px` }}
            >
                <DetailsPanel
                    item={selectedItems[0] || null}
                    isMultipleSelection={selectedItems.length > 1}
                />
            </div>
        </div>
    );
};

export default DetailsLayout;