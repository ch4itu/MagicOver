const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const html = fs.readFileSync(path.join(__dirname, '../index.html'), 'utf8');
const script = name => html.split('<script data-source="' + name + '">')[1].split('</script>')[0];
const map = new Function(script('farlands-map.js') + '\nreturn Farlands')();
const {boatFitsWater} = new Function(html.slice(html.indexOf('function boatHullTiles'), html.indexOf('function keepPlayerOnWalkableTile')) +
    '\nreturn {boatFitsWater}')();

const directions = [[1,0], [-1,0], [0,1], [0,-1]];
function reaches(from, to, passable, bounds) {
    const [left, top, right, bottom] = bounds;
    const width = right - left + 1;
    const key = (x,y) => (y-top)*width+x-left;
    const seen = new Uint8Array(width*(bottom-top+1));
    const queue = [from];
    seen[key(...from)] = 1;
    for (const [x,y] of queue) {
        if (x===to[0] && y===to[1]) return true;
        for (const [dx,dy] of directions) {
            const nx=x+dx, ny=y+dy;
            if (nx<left || nx>right || ny<top || ny>bottom || seen[key(nx,ny)]) continue;
            seen[key(nx,ny)]=1;
            if (passable(nx,ny)) queue.push([nx,ny]);
        }
    }
    return false;
}
function navigableWater(bounds) {
    const terrain = new map.Terrain();
    return (x,y) => {
        if (terrain.tileAt(x,y)!=='water') return false;
        // A five-tile fairway supports both orientations, including turning a station boat.
        for (let dy=-2;dy<=2;dy++) for(let dx=-2;dx<=2;dx++) {
            if (x+dx<bounds[0] || x+dx>bounds[2] || y+dy<bounds[1] || y+dy>bounds[3] ||
                terrain.tileAt(x+dx,y+dy)!=='water') return false;
        }
        return true;
    };
}
for (const [id, coordinates, sea, bounds, scale] of [
    ['amber',[7636,3866],[9450,4140],[7590,3650,9520,4360],1.35],
    ['lantern',[8500,8530],[8850,9150],[8250,8480,9070,9230],1.15],
    ['settlement-lantern-0',[8562,8568],[8850,9150],[8250,8480,9070,9230],0.85],
]) {
    const site = map.byId.get(id);
    assert.deepEqual([site.x,site.y], coordinates);
    const {shore,pier,water} = map.coastalDocks[id];
    const distance = Math.hypot(shore[0]-site.x,shore[1]-site.y);
    assert(distance<20, id+' landing is close to town');
    assert.equal(map.sample(...pier),'road',id+' pier');
    assert.equal(map.natural(...pier),'water',id+' pier extends into water');
    assert.notEqual(map.sample(...pier,true),'water',id+' pier cannot replace old navigable water');
    assert.notEqual(boatFitsWater(...water,map.sample,scale),null,id+' real hull clearance');
    const [wx,wy]=water;
    const clear=navigableWater(bounds);
    const local = [wx-8,wy-8,wx+8,wy+8];
    let channelEntry;
    for (let y=local[1];y<=local[3] && !channelEntry;y++) {
        for (let x=local[0];x<=local[2];x++) {
            if (clear(x,y) && reaches(water,[x,y],(tx,ty)=>map.sample(tx,ty)==='water' &&
                boatFitsWater(tx,ty,map.sample,scale)!==null,local)) {
                channelEntry=[x,y];
                break;
            }
        }
    }
    assert(channelEntry,id+' mooring opens to a fairway');
    assert(clear(...sea) && reaches(channelEntry,sea,clear,bounds),id+' fairway reaches open sea');
    assert(reaches(coordinates,shore,(x,y)=>!['water','mountain','lava'].includes(map.sample(x,y)),
        [site.x-25,site.y-25,site.x+25,site.y+25]),id+' walkable approach');
    console.log('PASS',id, 'dry approach, jetty, hull clearance and connected sea');
}
const pearl = map.byId.get('pearl');
assert.deepEqual([pearl.x,pearl.y],[8560,8565]);
assert(reaches([pearl.x,pearl.y],map.coastalDocks['settlement-lantern-0'].shore,
    (x,y)=>map.sample(x,y)!=='water',[8540,8544,8580,8590]));
console.log('PASS Pearl Banks walk to shared Shellwick landing');

const journeySource = script('farlands.js');
const migrate = new Function('Farlands', 'farlandsSaveVersion', journeySource.slice(
    journeySource.indexOf('    const startPosition'),
    journeySource.indexOf('    // R15.58: the boat')
) + '\nreturn migrateCoast')(map, 2);
const flooded = {x:8500,y:8562};
assert.equal(map.sample(flooded.x,flooded.y),'water');
assert.notEqual(map.sample(flooded.x,flooded.y,true),'water');
const save = {version:1, place:{...flooded,sailing:0}, horsePlace:flooded, horseOwned:true,
    boat:{parked:{x:8850,y:9150,heading:.25}}, pin:'pearl', spent:['rq-done-a-rest'], edits:[]};
const original = JSON.stringify(save);
const migrated = migrate(save);
assert.equal(migrated.version,2);
assert.equal(JSON.stringify(save),original);
assert.notDeepEqual(migrated.place,save.place);
assert.deepEqual(migrated.horsePlace,{x:migrated.place.x,y:migrated.place.y});
assert.notEqual(map.sample(migrated.place.x,migrated.place.y),'water');
assert.deepEqual(migrated.boat,save.boat);
assert.deepEqual(migrated.spent,save.spent);
const edited = {...save,edits:[[flooded.y*map.size+flooded.x,'road']]};
assert.deepEqual(migrate(edited).place,save.place);
assert.deepEqual(migrate(edited).edits,edited.edits);
const afloat = {...save,place:{x:8850,y:9150,sailing:15},horsePlace:{x:8850,y:9150}};
assert.deepEqual(migrate(afloat).place,afloat.place);
assert.deepEqual(migrate(afloat).horsePlace,afloat.horsePlace);
console.log('PASS v1 relocation, edit priority, boat and horse aboard, quest flags and input immutability');

for (const site of map.sites) {
    for (let y=site.y-9;y<=site.y+9;y++) for (let x=site.x-9;x<=site.x+9;x++) {
        if (map.sample(x,y,true)!=='water') {
            assert.notEqual(map.sample(x,y),'water',site.name+' dry building core');
        }
    }
}
for (const road of map.roads) {
    for (let i=1;i<road.length;i++) {
        const [x,y]=road[i-1], [endX,endY]=road[i];
        const length=Math.ceil(Math.hypot(endX-x,endY-y));
        for (let step=0;step<=length;step++) {
            const tx=Math.round(x+(endX-x)*step/length), ty=Math.round(y+(endY-y)*step/length);
            if (map.sample(tx,ty,true)==='road') assert.equal(map.sample(tx,ty),'road');
        }
    }
}
console.log('PASS dry settlement cores and existing road centerlines');
