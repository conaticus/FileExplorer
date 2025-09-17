import { invoke } from '@tauri-apps/api/core';
import { useEffect, useState, useCallback } from 'react';
import { useSftp } from '../providers/SftpProvider';

/**
 * Hook for managing file/folder preview functionality
 * @param {Function} getFocusedItem - Function that returns the currently focused file/folder object
 * @param {Function} navigateUp - Function to navigate up one row
 * @param {Function} navigateDown - Function to navigate down one row  
 * @param {Function} navigateLeft - Function to navigate left one column
 * @param {Function} navigateRight - Function to navigate right one column
 * @returns {Object} Object containing preview state and control functions
 */
export function usePreview(getFocusedItem, navigateUp = null, navigateDown = null, navigateLeft = null, navigateRight = null) {
  const [open, setOpen] = useState(false);
  const [payload, setPayload] = useState(null);
  const [isLoading, setIsLoading] = useState(false);
  const { isSftpPath, parseSftpPath } = useSftp();

  /**
   * Opens preview for the specified path
   * @param {string} path - Path to the file/folder to preview
   */
  const openPreview = useCallback(async (path) => {
    if (isLoading) return;

    setIsLoading(true);
    try {
      let previewPayload;
      
      // Check if this is an SFTP path
      if (isSftpPath(path)) {
        const parsed = parseSftpPath(path);
        if (parsed && parsed.connection) {
          // Use SFTP preview command
          previewPayload = await invoke('build_preview_sftp', {
            host: parsed.connection.host,
            port: parseInt(parsed.connection.port, 10),
            username: parsed.connection.username,
            password: parsed.connection.password,
            filePath: parsed.remotePath
          });
        } else {
          throw new Error('Invalid SFTP path or connection not found');
        }
      } else {
        // Use regular preview command for local files
        previewPayload = await invoke('build_preview', { path });
      }
      
      setPayload(previewPayload);
      setOpen(true);
    } catch (error) {
      console.error('Failed to build preview:', error);
      setPayload({
        kind: 'Error',
        name: path.split(/[/\\]/).pop() || 'Unknown',
        message: error instanceof Error ? error.message : 'Failed to generate preview'
      });
      setOpen(true);
    } finally {
      setIsLoading(false);
    }
  }, [isLoading, isSftpPath, parseSftpPath]);

  /**
   * Closes the preview modal
   */
  const closePreview = useCallback(() => {
    setOpen(false);
    setPayload(null);
  }, []);

  /**
   * Toggles preview for the currently focused item (files and folders)
   */
  const togglePreview = useCallback(() => {
    if (open) {
      closePreview();
      return;
    }

    const focusedItem = getFocusedItem();
    if (focusedItem && focusedItem.path) {
      openPreview(focusedItem.path);
    }
  }, [open, closePreview, openPreview, getFocusedItem]);

  // Keyboard event handler
  useEffect(() => {
    const onKey = (e) => {
      // Don't trigger if we're typing in an input
      if (e.target instanceof HTMLInputElement || e.target instanceof HTMLTextAreaElement) {
        return;
      }

      // Spacebar toggles preview
      if (e.key === ' ') {
        e.preventDefault();
        togglePreview();
      } 
      // Escape closes preview
      else if (e.key === 'Escape' && open) {
        e.preventDefault();
        closePreview();
      }
      // Arrow keys navigate when preview is open
      else if (open && ['ArrowUp', 'ArrowDown', 'ArrowLeft', 'ArrowRight'].includes(e.key)) {
        e.preventDefault();
        let navigationFunction = null;
        switch (e.key) {
          case 'ArrowUp':
            navigationFunction = navigateUp;
            break;
          case 'ArrowDown':
            navigationFunction = navigateDown;
            break;
          case 'ArrowLeft':
            navigationFunction = navigateLeft;
            break;
          case 'ArrowRight':
            navigationFunction = navigateRight;
            break;
        }
        if (navigationFunction) {
          navigationFunction();
          // Only preview the newly focused item after navigation
          setTimeout(() => {
            const focused = getFocusedItem();
            if (focused && focused.path) {
              openPreview(focused.path);
            } else {
              closePreview();
            }
          }, 0);
        }
      }
    };

    window.addEventListener('keydown', onKey);
    return () => window.removeEventListener('keydown', onKey);
  }, [open, togglePreview, closePreview, navigateUp, navigateDown, navigateLeft, navigateRight, getFocusedItem, openPreview]);

  return { 
    open, 
    payload, 
    isLoading,
    openPreview,
    closePreview, 
    togglePreview 
  };
}

// Also export as default for compatibility
export default usePreview;
