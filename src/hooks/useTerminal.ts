import { useEffect, useRef, useCallback, useState } from 'react';
import { Terminal } from 'xterm';
import { FitAddon } from 'xterm-addon-fit';
import { SearchAddon } from 'xterm-addon-search';
import { WebLinksAddon } from 'xterm-addon-web-links';
import { terminalInput, terminalResize } from '../services/ipc';
import { flushBuffer, setWriter, clearWriter, cleanupBuffer } from '../services/outputBuffer';

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
    console.log('[useTerminal] creating terminal, sessionId:', sessionId);
    if (!terminalRef.current) {
      console.log('[useTerminal] terminalRef.current is null, skipping');
      return;
    }

    console.log('[useTerminal] opening xterm...');
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
      scrollOnUserInput: false,
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
    console.log('[useTerminal] calling term.open()...');
    term.open(terminalRef.current);
    console.log('[useTerminal] term.open() done, scheduling fit...');

    // Fit to container
    setTimeout(() => {
      console.log('[useTerminal] fit timeout fired, sessionId:', sessionId);
      try {
        fitAddon.fit();
        console.log('[useTerminal] fit() succeeded');
      } catch (e) {
        console.error('[useTerminal] fit() error:', e);
      }
      setIsReady(true);
      // Flush any output buffered before terminal was ready, then register
      // a writer so future pushOutput calls go directly to xterm.
      if (sessionId) {
        console.log('[useTerminal] flushing buffer for', sessionId);
        const writer = (data: string) => { try { term.write(data); } catch (e) { console.error('[useTerminal] write error:', e); } };
        flushBuffer(sessionId, writer);
        setWriter(sessionId, writer);
        console.log('[useTerminal] writer registered');
      }
    }, 50);

    // Handle resize via window resize event — NOT ResizeObserver — to avoid
    // infinite loops where fit() → xterm re-renders → container changes →
    // ResizeObserver fires → fit() → repeat. Only the window size matters
    // for terminal dimensions; xterm output inside the container is irrelevant.
    let resizeTimeout: ReturnType<typeof setTimeout> | null = null;

    const handleWindowResize = () => {
      if (resizeTimeout) clearTimeout(resizeTimeout);
      resizeTimeout = setTimeout(() => {
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
      }, 150);
    };

    window.addEventListener('resize', handleWindowResize);

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
      console.log('[useTerminal] cleanup, sessionId:', sessionId);
      if (resizeTimeout) clearTimeout(resizeTimeout);
      window.removeEventListener('resize', handleWindowResize);
      document.removeEventListener('keydown', handleKeyDown);
      term.dispose();
      console.log('[useTerminal] term disposed');
      terminalInstance.current = null;
      fitAddonInstance.current = null;
      searchAddonInstance.current = null;
      setIsReady(false);
      if (sessionId) { clearWriter(sessionId); cleanupBuffer(sessionId); }
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  // Handle data events from terminal input
  useEffect(() => {
    const term = terminalInstance.current;
    if (!term) {
      console.log('[useTerminal] onData effect: term not ready, skipping');
      return;
    }

    console.log('[useTerminal] registering onData handler, sessionId:', sessionId);
    const disposable = term.onData((data: string) => {
      if (sessionId) {
        console.log('[useTerminal] onData:', JSON.stringify(data).substring(0, 80));
        terminalInput(sessionId, data);
      }
      onData?.(data);
    });

    return () => {
      console.log('[useTerminal] removing onData handler');
      disposable.dispose();
    };
  }, [sessionId, onData]);

  // Fit terminal when search visibility changes
  useEffect(() => {
    console.log('[useTerminal] search visible changed, re-fitting...');
    setTimeout(() => {
      try {
        fitAddonInstance.current?.fit();
      } catch (e) {
        console.error('[useTerminal] search fit error:', e);
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
