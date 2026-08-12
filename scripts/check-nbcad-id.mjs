import assert from 'node:assert/strict';
import { Domain, fromPreimage, isNbcadUuid, isValidSessionId } from './nbcad-id.mjs';

const golden = fromPreimage(Domain.Session, 'golden');
assert.equal(golden, '01732db8-694c-886c-87d8-c2c64537d673');
assert.equal(golden.startsWith('01'), true);
assert.equal(isNbcadUuid(golden), true);
assert.equal(isValidSessionId('My Document'), false);
assert.equal(isValidSessionId('123e4567-e89b-42d3-a456-426614174000'), true);
assert.equal(isNbcadUuid('123e4567-e89b-42d3-a456-426614174000'), false);
console.log('nbcad-id JS packing matches crates/id golden');
