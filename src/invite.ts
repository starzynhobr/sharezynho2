const INVITE_PREFIX = 'sharezynho://';

export const buildInvite = (address: string, room: string): string =>
  `${INVITE_PREFIX}${address}/${room}`;

export const parseInvite = (text: string): { address: string; room: string } | null => {
  const match = text.trim().match(/^(?:sharezynho:\/\/)?([^\s/]+)\/([A-Za-z0-9-]+)\/?$/i);
  return match ? { address: match[1], room: match[2].toUpperCase() } : null;
};
