const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const crypto = require('node:crypto');

const html = fs.readFileSync(process.argv[2] || path.join(__dirname, '../index.html'), 'utf8');
const script = name => html.split('<script data-source="' + name + '">')[1].split('</script>')[0];
const Farlands = new Function(script('farlands-map.js') + '\nreturn Farlands')();

// Use gameplay terrain, including roads and settlement overrides. A nearby tile
// of water is only a proximity check. It does not certify a usable mooring.
const coastalIds = ['amber', 'lantern', 'settlement-lantern-0', 'pearl', 'wreck', 'beacon', 'beach'];
const sites = Farlands.sites.filter(site =>
    site.waterfront || coastalIds.includes(site.id) || Farlands.waterwayPorts.some(port => port.id === site.id));

function shorelineDistance(site, limit) {
    let best = Infinity;
    for (let dy = -limit; dy <= limit; dy++) {
        for (let dx = -limit; dx <= limit; dx++) {
            const distance = dx * dx + dy * dy;
            if (distance > limit * limit || distance >= best) continue;
            if (Farlands.sample(site.x + dx, site.y + dy) === 'water') best = distance;
        }
    }
    return Number.isFinite(best) ? Math.round(Math.sqrt(best) * 10) / 10 : null;
}

const geography = Object.fromEntries(sites.map(site => [site.id, {
    name: site.name,
    withinSettlementRadius: shorelineDistance(site, site.radius),
    waterWithin450: shorelineDistance(site, 450),
}]));
assert.equal(sites.length, 28);
assert.equal(Farlands.byId.size, Farlands.sites.length);
assert.notEqual(geography.rest.withinSettlementRadius, null);
assert.notEqual(geography.alder.withinSettlementRadius, null);

if (process.argv.includes('--require-waterfront')) {
    const inland = sites.filter(site => geography[site.id].withinSettlementRadius === null).map(site => site.name);
    assert.deepEqual(inland, [], 'Named waterfront sites without water inside their authored radius');
}

const farlandsSource = script('farlands.js');
const restoreJourney = new Function('Farlands', farlandsSource.slice(
    farlandsSource.indexOf('    const startPosition'),
    farlandsSource.indexOf('    // R15.58: the boat')
) + '\nreturn newJourney')(Farlands);
const fixture = {
    version: 1,
    station: 'settlement-lantern-0',
    pin: 'pearl',
    place: {x: 8562, y: 8568, sailing: 0, room: {id: 'fixture-room', x: 4, y: 5}},
    horseOwned: true,
    horsePlace: {x: 8563, y: 8568},
    visited: ['pearl'],
    spent: ['rq-taken-b-settlement-lantern-0', 'rq-done-a-rest'],
    edits: [[85688562, 'road']],
};
if (!process.argv.includes('--baseline')) {
    const restored = restoreJourney(fixture);
    assert.equal(restored.station, fixture.station);
    assert.equal(restoreJourney({...fixture, station: 'pearl'}).station, 'rest');
    assert.equal(restoreJourney({...fixture, station: 'unknown'}).station, 'rest');
    assert.deepEqual(restored.place, fixture.place);
    assert.deepEqual(restored.horsePlace, fixture.horsePlace);
    assert.deepEqual([...restored.terrain.edits], fixture.edits);
    assert.deepEqual([...restored.spent], fixture.spent);
    assert.deepEqual([...restored.visited], fixture.visited);
    assert.equal(restored.pin, fixture.pin);
}
console.log(JSON.stringify({
    sha256: crypto.createHash('sha256').update(html).digest('hex'),
    geography,
    v1JourneyFixture: process.argv.includes('--baseline') ? 'skipped for baseline' : 'passed',
}, null, 2));
