import assert from 'node:assert/strict';
import test from 'node:test';
import { desktopLink, isMap } from '../src/desktopLink.ts';

test('launches only a selected .mm file with validated IDs', () => {
  assert.equal(isMap('日本語.MM'), true);
  assert.equal(isMap('map.mm.txt'), false);
  const tenant = '52d13ec3-244a-44b6-9072-19cfa0eaac05';
  const url = new URL(desktopLink(tenant, 'b!drive', '01ITEM'));
  assert.equal(url.protocol, 'mindglider:');
  assert.equal(url.searchParams.get('tenant'), tenant);
  assert.equal(url.searchParams.get('drive'), 'b!drive');
  assert.throws(() => desktopLink(tenant, 'b!drive&token=test', '01ITEM'));
});
