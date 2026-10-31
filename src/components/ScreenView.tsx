import { useCallback, useEffect, useRef, useState, type PointerEvent } from 'react';
import { Maximize, Minimize } from 'lucide-react';
import { playVideoStream } from '../videoStream';

const MAX_ZOOM = 6;
const ZOOM_STEP = 1.15;
const RESET_VIEW = { scale: 1, x: 0, y: 0 };

interface ScreenViewProps {
  src: string;
  label: string;
}

/** Keeps the zoomed image covering the whole viewport. */
const clampOffset = (offset: number, size: number, scale: number) =>
  Math.min(0, Math.max(size * (1 - scale), offset));

/** Shared screen with in-app fullscreen, Ctrl + scroll zoom and drag to pan. */
export const ScreenView: React.FC<ScreenViewProps> = ({ src, label }) => {
  const containerRef = useRef<HTMLElement>(null);
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const dragRef = useRef<{ pointerX: number; pointerY: number; x: number; y: number } | null>(null);
  const [isFullscreen, setIsFullscreen] = useState(false);
  const [view, setView] = useState(RESET_VIEW);

  const toggleFullscreen = useCallback(() => {
    setIsFullscreen((current) => !current);
    setView(RESET_VIEW);
  }, []);

  useEffect(() => {
    const canvas = canvasRef.current;
    return canvas ? playVideoStream(src, canvas) : undefined;
  }, [src]);

  useEffect(() => {
    const container = containerRef.current;
    if (!container) return;

    // React wheel listeners are passive, so they cannot stop the page zoom
    const handleWheel = (e: WheelEvent) => {
      if (!e.ctrlKey) return;
      e.preventDefault();
      const rect = container.getBoundingClientRect();
      const mouseX = e.clientX - rect.left;
      const mouseY = e.clientY - rect.top;
      setView((current) => {
        const factor = e.deltaY < 0 ? ZOOM_STEP : 1 / ZOOM_STEP;
        const scale = Math.min(MAX_ZOOM, Math.max(1, current.scale * factor));
        const ratio = scale / current.scale;
        return {
          scale,
          x: clampOffset(mouseX - (mouseX - current.x) * ratio, rect.width, scale),
          y: clampOffset(mouseY - (mouseY - current.y) * ratio, rect.height, scale),
        };
      });
    };

    container.addEventListener('wheel', handleWheel, { passive: false });
    return () => container.removeEventListener('wheel', handleWheel);
  }, []);

  useEffect(() => {
    if (!isFullscreen) return;
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === 'Escape') toggleFullscreen();
    };
    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [isFullscreen, toggleFullscreen]);

  const handlePointerDown = (e: PointerEvent) => {
    if (view.scale === 1) return;
    e.currentTarget.setPointerCapture(e.pointerId);
    dragRef.current = { pointerX: e.clientX, pointerY: e.clientY, x: view.x, y: view.y };
  };

  const handlePointerMove = (e: PointerEvent) => {
    const drag = dragRef.current;
    const container = containerRef.current;
    if (!drag || !container) return;
    const { width, height } = container.getBoundingClientRect();
    setView((current) => ({
      ...current,
      x: clampOffset(drag.x + e.clientX - drag.pointerX, width, current.scale),
      y: clampOffset(drag.y + e.clientY - drag.pointerY, height, current.scale),
    }));
  };

  const handlePointerUp = () => {
    dragRef.current = null;
  };

  const FullscreenIcon = isFullscreen ? Minimize : Maximize;
  const fullscreenLabel = isFullscreen ? 'Sair da tela cheia' : 'Tela cheia';

  return (
    <section
      ref={containerRef}
      onPointerDown={handlePointerDown}
      onPointerMove={handlePointerMove}
      onPointerUp={handlePointerUp}
      onPointerCancel={handlePointerUp}
      onDoubleClick={toggleFullscreen}
      className={`overflow-hidden select-none ${
        isFullscreen ? 'fixed inset-0 z-50 bg-black' : 'relative flex-1 min-h-0 rounded-2xl bg-black/40 border border-border-subtle'
      } ${view.scale > 1 ? 'cursor-grab active:cursor-grabbing' : ''}`}
    >
      <canvas
        ref={canvasRef}
        aria-label={label}
        style={{
          transform: `translate(${view.x}px, ${view.y}px) scale(${view.scale})`,
          transformOrigin: '0 0',
        }}
        className="w-full h-full object-contain"
      />
      <span className="absolute top-3 left-3 rounded-lg bg-root/85 px-2.5 py-1.5 text-xs font-semibold text-white">
        {label}
      </span>
      <button
        onClick={toggleFullscreen}
        onPointerDown={(e) => e.stopPropagation()}
        onDoubleClick={(e) => e.stopPropagation()}
        title={fullscreenLabel}
        aria-label={fullscreenLabel}
        className="absolute top-3 right-3 w-9 h-9 rounded-lg bg-root/85 hover:bg-control text-white flex items-center justify-center transition-colors cursor-pointer"
      >
        <FullscreenIcon className="w-4 h-4" />
      </button>
    </section>
  );
};
