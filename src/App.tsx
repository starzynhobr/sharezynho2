import { useEffect } from 'react';
import { api } from './api';
import { useRoomStore } from './store/roomStore';
import { RoomsSidebar } from './components/RoomsSidebar';
import { MainCanvas } from './components/MainCanvas';
import { JoinCreateModal, InviteModal, MonitorPickerModal } from './components/Dialogs';

export function App() {
  const init = useRoomStore((s) => s.init);

  // Dialog state needed for modal mounts
  const isJoinCreateOpen = useRoomStore((s) => s.isJoinCreateOpen);
  const joinCreateIsModeCreate = useRoomStore((s) => s.joinCreateIsModeCreate);
  const networkInfo = useRoomStore((s) => s.networkInfo);
  const closeJoinCreate = useRoomStore((s) => s.closeJoinCreate);
  const joinCreateSubmit = useRoomStore((s) => s.joinCreateSubmit);

  const isInviteOpen = useRoomStore((s) => s.isInviteOpen);
  const hostAddress = useRoomStore((s) => s.hostAddress);
  const currentRoom = useRoomStore((s) => s.currentRoom);
  const closeInvite = useRoomStore((s) => s.closeInvite);

  const isMonitorPickerOpen = useRoomStore((s) => s.isMonitorPickerOpen);
  const monitors = useRoomStore((s) => s.monitors);
  const closeMonitorPicker = useRoomStore((s) => s.closeMonitorPicker);
  const selectMonitor = useRoomStore((s) => s.selectMonitor);

  useEffect(() => {
    // Reveal window once React mounts
    api.showWindow();

    // Initialize network info and listeners with race-condition-safe cleanup
    const cleanup = init();
    return cleanup;
  }, [init]);

  return (
    <div className="flex w-full h-full bg-root overflow-hidden select-none">
      {currentRoom && <RoomsSidebar />}

      {/* 3. Main Stage Canvas */}
      <MainCanvas />

      {/* Dialogs */}
      <JoinCreateModal
        isOpen={isJoinCreateOpen}
        initialModeIsCreate={joinCreateIsModeCreate}
        detectedIp={networkInfo?.preferred_ip || '127.0.0.1'}
        defaultPort={networkInfo?.tcp_port || 9765}
        onClose={closeJoinCreate}
        onSubmit={joinCreateSubmit}
      />

      <InviteModal
        isOpen={isInviteOpen}
        hostAddress={hostAddress}
        room={currentRoom || ''}
        onClose={closeInvite}
      />

      <MonitorPickerModal
        isOpen={isMonitorPickerOpen}
        monitors={monitors}
        onClose={closeMonitorPicker}
        onSelectMonitor={selectMonitor}
      />
    </div>
  );
}

export default App;
