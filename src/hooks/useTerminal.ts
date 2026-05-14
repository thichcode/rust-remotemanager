import { useEffect, useRef, useCallback, useState } from 'react';
import { Terminal } from 'xterm';
import { FitAddon } from 'xterm-addon-fit';
import { SearchAddon } from 'xterm-addon-search';
import { WebLinksAddon } from 'xterm-addon-web-links';
import { terminalInput, terminalResize, listenToTerminalOutput } from '../services/ipc';

interface UseTerminalOptions {
  sessionId: string;
  onData?: (data: string) => void;
}

interface UseTerminalReturn {
  terminalRef: React.MutableRefObject<HTMLDivElement | null>;
  terminal: Terminal | null;
  fitAddon: FitAddon | null;
  searchAddon: SearchAddon | null;
  isReady: boolean;
  searchVisible: boolean;
  searchText: string;
  setSearchText: (text: string) => void;
  focusSearch: () => void;
  closeSearch: () => void;
  findNext: () => void;
  findPrevious: () => void;
}

export function useTerminal({ sessionId, onData }: UseTerminalOptions): UseTerminalReturn {
  const terminalRef = useRef<HTMLDivElement>(null!);
  const terminalInstance = useRef<Terminal | null>(null);
  const fitAddonInstance = useRef<FitAddon | null>(null);
  const searchAddonInstance = useRef<SearchAddon | null>(null);
  const [isReady, setIsReady] = useState(false);
  const [searchVisible, setSearchVisible] = useState(false);
  const [searchText, setSearchText] = useState('');

  // Create terminal
  useEffect(() => {
    if (!terminalRef.current) return;

    const term = new Terminal({
      cursorBlink: true,
      cursorStyle: 'block',
      fontSize: 14,
      fontFamily: "'JetBrains Mono', 'Fira Code', monospace",
      theme: {
        background: '#0d1117',
        foreground: '#c9d1d9',
        cursor: '#58a6ff',
        selectionBackground: '#264f78',
        black: '#0d1117',
        red: '#ff7b72',
        green: '#3fb950',
        yellow: '#d29922',
        blue: '#58a6ff',
        magenta: '#bc8cff',
        cyan: '#39c5cf',
        white: '#b1bac4',
        brightBlack: '#484f58',
        brightRed: '#ffa198',
        brightGreen: '#56d364',
        brightYellow: '#e3b341',
        brightBlue: '#79c0ff',
        brightMagenta: '#d2a8ff',
        brightCyan: '#56d4dd',
        brightWhite: '#f0f6fc',
      },
      allowTransparency: false,
      scrollback: 5000,
      convertEol: true,
      disableStdin: false,
      cols: 80,
      rows: 24,
    });

    const fitAddon = new FitAddon();
    term.loadAddon(fitAddon);
    fitAddonInstance.current = fitAddon;

    const searchAddon = new SearchAddon();
    term.loadAddon(searchAddon);
    searchAddonInstance.current = searchAddon;

    const webLinksAddon = new WebLinksAddon();
    term.loadAddon(webLinksAddon);

    terminalInstance.current = term;

    // Open terminal in container
    term.open(terminalRef.current);

    // Fit to container
    setTimeout(() => {
      try {
        fitAddon.fit();
      } catch {
        // Ignore fit errors
      }
      setIsReady(true);
    }, 50);

    // Handle resize
    const resizeObserver = new ResizeObserver(() => {
      try {
        fitAddon.fit();
        if (sessionId) {
          const dims = fitAddon.proposeDimensions();
          if (dims) {
            terminalResize(sessionId, dims.cols, dims.rows);
          }
        }
      } catch {
        // Ignore resize errors
      }
    });

    if (terminalRef.current) {
      resizeObserver.observe(terminalRef.current);
    }

    // Keyboard shortcut for search (Ctrl+F / Cmd+F)
    const handleKeyDown = (e: KeyboardEvent) => {
      if ((e.ctrlKey || e.metaKey) && e.key === 'f') {
        e.preventDefault();
        setSearchVisible((prev) => !prev);
      }
      if (e.key === 'Escape') {
        setSearchVisible(false);
      }
    };

    document.addEventListener('keydown', handleKeyDown);

    // Cleanup
    return () => {
      resizeObserver.disconnect();
      document.removeEventListener('keydown', handleKeyDown);
      term.dispose();
      terminalInstance.current = null;
      fitAddonInstance.current = null;
      searchAddonInstance.current = null;
      setIsReady(false);
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  // Handle data events from terminal input
  useEffect(() => {
    const term = terminalInstance.current;
    if (!term) return;

    const disposable = term.onData((data: string) => {
      if (sessionId) {
        terminalInput(sessionId, data);
      }
      onData?.(data);
    });

    return () => {
      disposable.dispose();
    };
  }, [sessionId, onData]);

  // Listen for terminal output events
  useEffect(() => {
    if (!sessionId) return;

    let unlisten: (() => void) | undefined;

    const setup = async () => {
      try {
        unlisten = await listenToTerminalOutput(sessionId, (event) => {
          const term = terminalInstance.current;
          if (term) {
            term.write(event.data);
          }
        });
      } catch (err) {
        console.error('Failed to listen to terminal output:', err);
      }
    };

    setup();

    return () => {
      unlisten?.();
    };
  }, [sessionId]);

  // Fit terminal when search visibility changes
  useEffect(() => {
    setTimeout(() => {
      try {
        fitAddonInstance.current?.fit();
      } catch {
        // ignore
      }
    }, 100);
  }, [searchVisible]);

  const focusSearch = useCallback(() => {
    setSearchVisible(true);
  }, []);

  const closeSearch = useCallback(() => {
    setSearchVisible(false);
    setSearchText('');
    terminalInstance.current?.focus();
  }, []);

  const findNext = useCallback(() => {
    if (searchText) {
      searchAddonInstance.current?.findNext(searchText);
    }
  }, [searchText]);

  const findPrevious = useCallback(() => {
    if (searchText) {
      searchAddonInstance.current?.findPrevious(searchText);
    }
  }, [searchText]);

  return {
    terminalRef: terminalRef as React.MutableRefObject<HTMLDivElement | null>,
    terminal: terminalInstance.current,
    fitAddon: fitAddonInstance.current,
    searchAddon: searchAddonInstance.current,
    isReady,
    searchVisible,
    searchText,
    setSearchText,
    focusSearch,
    closeSearch,
    findNext,
    findPrevious,
  };
}
