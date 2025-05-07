import React, { useState, useRef, useEffect } from 'react';
import { useFileSystem } from '../providers/FileSystemProvider';
import DetailsPanel from '../components/explorer/DetailsPanel';
import './detailsLayout.css';

const DetailsLayout = ({ children }) => {
    const { selectedItems } = useFileSystem();
    const [panelWidth, setPanelWidth] = useState(300); // Default width
    const [isResizing, setIsResizing] = useState(false);
    const containerRef = useRef(null);
    const startXRef = useRef(0);
    const startWidthRef = useRef(0);

    // Handle resize start
    const handleResizeStart = (e) => {
        setIsResizing(true);
        startXRef.current = e.clientX;
        startWidthRef.current = panelWidth;

        // Prevent text selection during resize
        document.body.style.userSelect = 'none';
        document.body.style.cursor = 'col-resize';
    };

    // Handle resize during mouse move
    useEffect(() => {
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