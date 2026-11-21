const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const {performance} = require('node:perf_hooks');
const html = fs.readFileSync(path.join(__dirname, '../index.html'), 'utf8');
const baseline = fs.readFileSync(process.argv[2], 'utf8');
const script = (source, name) => source.split('<script data-source="' + name + '">')[1].split('</script>')[0];
const Farlands = new Function(script(html, 'farlands-map.js') + '\nreturn Farlands')();
const flags = new Set();
const state = {player: {x: 8562, y: 8568}, inBattle: false, sailingMoves: 0};
let pinned;
const makeQuests = source => new Function('Farlands', 'ebFarlands', 'sharedWorld', 'buildingInterior',
    'gameState', 'document', 'ebDisplayNameOf', 'showFloatingText',
    script(source, 'rpg-local-quests.js').replace('return {plan,accept,', 'return {activePlans,plan,accept,') +
    '\nreturn ebRpgQuests')(
    Farlands,
    {surface: () => true, storyFlag: key => flags.has(key), markRpgQuest: key => !flags.has(key) && !!flags.add(key),
        refreshStoryHint() {}, pin: id => { pinned = id; }},
    {active: () => false}, null, state, {getElementById: () => null}, name => name, () => {}
);
const old = makeQuests(baseline);
const current = makeQuests(html);
const authored = ['rest', 'settlement-hearth-0', 'settlement-lantern-0'];
for (const site of Farlands.sites.filter(site => site.station)) {
    for (const kind of 'abc') {
        const a = old.plan(site, kind);
        const b = current.plan(site, kind);
        assert.deepEqual([b.target.id, b.giver, b.reward], [a.target.id, a.giver, a.reward]);
        if (authored.includes(site.id)) {
            assert.notEqual(b.title, a.title);
            assert(b.description.endsWith(a.description));
        } else {
            assert.deepEqual(b, a);
        }
    }
}
flags.add('rq-taken-b-settlement-lantern-0');
flags.add('rq-taken-a-rest');
flags.add('rq-done-a-rest');
assert.deepEqual(current.activePlans().map(q => q.site.id), ['settlement-lantern-0']);
const shellwick = Farlands.byId.get('settlement-lantern-0');
const pearl = Farlands.byId.get('pearl');
current.onLocation(shellwick);
assert(!flags.has('rq-step1-b-settlement-lantern-0'), 'Shellwick does not count as Pearl Banks');
Object.assign(state.player, {x: pearl.x, y: pearl.y});
current.onLocation(pearl);
assert(flags.has('rq-step1-b-settlement-lantern-0'));
const soundings=current.plan(shellwick,'d');
assert.equal(soundings.title,'Sound the Lantern Channel');
assert.equal(current.plan(Farlands.start,'d'),null);
assert(current.accept(soundings));
assert.equal(pinned,shellwick.id);
function checkMooring(id, sailingMoves) {
    const [x,y]=Farlands.coastalDocks[id].water;
    Object.assign(state.player,{x,y});
    state.sailingMoves=sailingMoves;
    current.onLocation(null);
}
checkMooring('lantern',15);
assert(!flags.has('rq-step2-d-settlement-lantern-0'),'arrival before departure does not count');
checkMooring(shellwick.id,0);
assert(!flags.has('rq-step1-d-settlement-lantern-0'),'standing at the coast does not count');
checkMooring(shellwick.id,15);
assert(flags.has('rq-step1-d-settlement-lantern-0'));
assert.equal(pinned,'lantern');
checkMooring('lantern',0);
assert(!flags.has('rq-step2-d-settlement-lantern-0'),'overland arrivals do not count');
checkMooring('lantern',15);
assert(flags.has('rq-step2-d-settlement-lantern-0'));
assert.equal(pinned,shellwick.id);
flags.add('rq-done-d-settlement-lantern-0');
assert(!current.activePlans().some(q=>q.kind==='d'));
assert(!current.accept(soundings));
console.log('PASS coastal quest respects afloat state, order, pinning and completed flags');
const creek = Farlands.creekPilotage;
const [shoreX, shoreY] = creek.board;
const [reachX, reachY] = creek.reach;
const board = Farlands.propAt(shoreX, shoreY);
assert.equal(board.name, creek.name);
assert.equal(Farlands.propAt(shoreX, shoreY, false), undefined);
assert.notEqual(Farlands.sample(shoreX, shoreY), 'water');
assert.equal(Farlands.sample(reachX, reachY), 'water');
const pilotage = current.plan(Farlands.start, 'g');
assert.equal(pilotage.giver, 'Hale, the Wheelwright');
assert.equal(current.plan(shellwick, 'g'), null);
assert(current.accept(pilotage));
Object.assign(state.player, {x:reachX,y:reachY});
state.sailingMoves=15;
current.onLocation(null);
assert(!flags.has('rq-step2-g-rest'), 'sailing before reading the board does not count');
assert.equal(current.onProp(board), '');
Object.assign(state.player, {x:shoreX,y:shoreY});
state.sailingMoves=0;
assert.match(current.onProp(board), /Sounding started/);
Object.assign(state.player, {x:reachX,y:reachY});
current.onLocation(null);
assert(!flags.has('rq-step2-g-rest'), 'a coordinate alone does not count as sailing');
state.sailingMoves=15;
current.onLocation(null);
assert(flags.has('rq-step2-g-rest'));
console.log('PASS Mill Creek board range, solo scenery, ordered progress and afloat gate');

function measure(quests) {
    for (let i = 0; i < 100; i++) quests.activePlans();
    const start = performance.now();
    for (let i = 0; i < 1000; i++) quests.activePlans();
    return Math.round(performance.now() - start);
}
console.log(JSON.stringify({result: 'PASS', iterations: 1000, oldMs: measure(old), candidateMs: measure(current)}));
