import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';
import tailwindcss from '@tailwindcss/vite';
import path from 'path';

export default defineConfig({
  base: './',
  // Prevent Vite from obscuring Rust compiler errors
  clearScreen: false,
  plugins: [
    react(),
    tailwindcss(),
  ],
  resolve: {
    alias: {
      '@': path.resolve(__dirname, './src'),
    },
  },
  server: {
    port: 5173,
    strictPort: true,
    watch: {
      // Don't watch Rust backend files to avoid wasted CPU and reload loops
      ignored: ['**/src-tauri/**'],
    },
    warmup: {
      clientFiles: [
        './index.html',
        './src/main.tsx',
        './src/App.tsx',
        './src/index.css',
        './src/api.ts',
        './src/types.ts',
        './src/components/NavigationRail.tsx',
        './src/components/RoomsSidebar.tsx',
        './src/components/MainCanvas.tsx',
        './src/components/Dialogs.tsx',
        './src/store/roomStore.ts',
      ],
    },
  },
  envPrefix: ['VITE_', 'TAURI_'],
  build: {
    // Target modern Chromium WebView2 in Windows for smaller, faster code
    target: process.env.TAURI_PLATFORM === 'windows' ? 'chrome105' : 'safari13',
    minify: !process.env.TAURI_DEBUG ? 'esbuild' : false,
    sourcemap: !!process.env.TAURI_DEBUG,
  },
  optimizeDeps: {
    include: [
      'react',
      'react-dom',
      'zustand',
      'lucide-react',
      'framer-motion',
      '@tauri-apps/api/core',
      '@tauri-apps/api/event',
      '@tauri-apps/api/window',
      'clsx',
      'tailwind-merge',
    ],
  },
});

