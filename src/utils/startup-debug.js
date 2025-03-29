// Diese Datei enthält Hilfsfunktionen zum Debuggen von Startup-Problemen

/**
 * Fügt ein debug-Element auf der Seite hinzu, das Fehler anzeigt
 * und hilft, Probleme zu diagnostizieren
 */
export function addDebugElement() {
    try {
        // Erstelle ein Container-Element
        const debugContainer = document.createElement('div');
        debugContainer.style.position = 'fixed';
        debugContainer.style.top = '10px';
        debugContainer.style.left = '10px';
        debugContainer.style.right = '10px';
        debugContainer.style.padding = '15px';
        debugContainer.style.background = 'rgba(255, 0, 0, 0.8)';
        debugContainer.style.color = 'white';
        debugContainer.style.fontFamily = 'monospace';
        debugContainer.style.fontSize = '14px';
        debugContainer.style.zIndex = '9999';
        debugContainer.style.borderRadius = '5px';
        debugContainer.style.boxShadow = '0 0 10px rgba(0,0,0,0.5)';
        debugContainer.style.overflow = 'auto';
        debugContainer.style.maxHeight = '80vh';

        // Hinzufügen eines Titels
        const title = document.createElement('h2');
        title.textContent = 'Rendering-Diagnose';
        title.style.margin = '0 0 10px 0';
        debugContainer.appendChild(title);

        // Systeminformationen hinzufügen
        const info = document.createElement('div');
        info.innerHTML = `
      <p><strong>Browser:</strong> ${navigator.userAgent}</p>
      <p><strong>URL:</strong> ${window.location.href}</p>
      <p><strong>Zeit:</strong> ${new Date().toLocaleString()}</p>
    `;
        debugContainer.appendChild(info);

        // DOM-Struktur analysieren
        const domInfo = document.createElement('div');
        domInfo.innerHTML = `
      <h3>DOM-Struktur:</h3>
      <p><strong>Root ID vorhanden:</strong> ${document.getElementById('root') ? 'Ja' : 'Nein'}</p>
      <p><strong>Body-Inhalt:</strong> ${document.body.children.length} Kind-Elemente</p>
    `;
        debugContainer.appendChild(domInfo);

        // Style-Informationen
        const styleInfo = document.createElement('div');
        styleInfo.innerHTML = `
      <h3>Style-Informationen:</h3>
      <p><strong>Style-Elemente:</strong> ${document.querySelectorAll('style').length}</p>
      <p><strong>Stylesheets:</strong> ${document.styleSheets.length}</p>
    `;
        debugContainer.appendChild(styleInfo);

        // Schaltfläche zum Schließen des Debug-Elements
        const closeButton = document.createElement('button');
        closeButton.textContent = 'Schließen';
        closeButton.style.marginTop = '10px';
        closeButton.style.padding = '5px 10px';
        closeButton.style.cursor = 'pointer';
        closeButton.onclick = () => document.body.removeChild(debugContainer);
        debugContainer.appendChild(closeButton);

        // Dem Body hinzufügen
        document.body.appendChild(debugContainer);

        console.log('Debug-Element wurde erfolgreich hinzugefügt');
        return true;
    } catch (error) {
        console.error('Fehler beim Erstellen des Debug-Elements:', error);
        return false;
    }
}

/**
 * Überprüft die grundlegende App-Struktur und gibt Feedback im Konsolenfenster
 */
export function checkAppStructure() {
    console.log('Überprüfe App-Struktur...');

    // Überprüfe Root-Element
    const root = document.getElementById('root');
    console.log('Root-Element:', root);

    if (root) {
        console.log('Root-Inhalt:', root.innerHTML);
    }

    // Überprüfe, ob React-Skripte geladen wurden
    const scripts = Array.from(document.querySelectorAll('script')).map(script => script.src);
    console.log('Geladene Skripte:', scripts);

    // Überprüfe, ob die App-Container-Klasse existiert
    const appContainers = document.querySelectorAll('.app-container');
    console.log('App-Container gefunden:', appContainers.length);

    return {
        rootExists: !!root,
        rootHasContent: root && root.innerHTML.trim().length > 0,
        scriptsLoaded: scripts.length > 0,
        appContainersFound: appContainers.length > 0
    };
}

/**
 * Fügt einen einfachen Platzhalter-Inhalt zur Seite hinzu
 * als Fallback, wenn die App nicht richtig lädt
 */
export function addFallbackContent() {
    try {
        const rootElement = document.getElementById('root');

        if (!rootElement || rootElement.children.length === 0) {
            // Wenn das Root-Element leer ist, füge Fallback-Inhalt hinzu
            const fallbackContent = document.createElement('div');
            fallbackContent.style.padding = '20px';
            fallbackContent.style.maxWidth = '600px';
            fallbackContent.style.margin = '40px auto';
            fallbackContent.style.fontFamily = 'system-ui, -apple-system, sans-serif';
            fallbackContent.style.lineHeight = '1.5';
            fallbackContent.style.color = '#333';

            fallbackContent.innerHTML = `
        <h1 style="color: #0078d4; margin-bottom: 20px;">Fast File Explorer</h1>
        <p>Es scheint ein Problem beim Laden der Anwendung zu geben. Hier sind einige Schritte zur Fehlerbehebung:</p>
        <ol>
          <li>Überprüfen Sie die Browser-Konsole auf Fehlermeldungen (F12)</li>
          <li>Stellen Sie sicher, dass JavaScript aktiviert ist</li>
          <li>Versuchen Sie, die Seite neu zu laden (F5)</li>
          <li>Starten Sie die Anwendung mit "cargo tauri dev" neu</li>
        </ol>
        <div style="margin-top: 20px; padding: 15px; background: #f5f5f5; border-radius: 5px;">
          <strong>Technische Informationen:</strong>
          <pre style="margin-top: 10px; overflow: auto; white-space: pre-wrap;">${JSON.stringify({
                timestamp: new Date().toISOString(),
                userAgent: navigator.userAgent,
                url: window.location.href,
                rootElement: !!rootElement
            }, null, 2)}</pre>
        </div>
      `;

            if (rootElement) {
                rootElement.appendChild(fallbackContent);
            } else {
                document.body.appendChild(fallbackContent);
            }

            console.log('Fallback-Inhalt wurde hinzugefügt');
            return true;
        }

        return false;
    } catch (error) {
        console.error('Fehler beim Hinzufügen des Fallback-Inhalts:', error);
        return false;
    }
}