const identifier = /^[A-Za-z0-9!_-]{1,512}$/;

export function isMap(name: unknown): boolean {
  return typeof name === 'string' && name.toLowerCase().endsWith('.mm');
}

export function desktopLink(tenant: string, drive: string, item: string): string {
  if (!/^[0-9a-f]{8}(?:-[0-9a-f]{4}){3}-[0-9a-f]{12}$/i.test(tenant) ||
      !identifier.test(drive) || !identifier.test(item)) {
    throw new Error('Invalid SharePoint document identifiers');
  }
  return 'mindglider://sharepoint?' + new URLSearchParams({ tenant, drive, item }).toString();
}
