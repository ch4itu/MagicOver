const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const vm = require('node:vm');

const source = file => fs.readFileSync(file, 'utf8');
const script = (html, name) => html.split('<script data-source="' + name + '">')[1].split('</script>')[0];
const current = source(process.env.EB_SOURCE || path.join(__dirname, '../index.html'));
const baseline = source(process.argv[2]);
const context = vm.createContext({});
vm.runInContext(script(current, 'three.min.js'), context);
const THREE = context.THREE;
const art = html => new Function('THREE', script(html, 'gui-models.js') + '\nreturn EternalBlissArt')(THREE);
const old = art(baseline);
const candidate = art(current);

function surfaces(model) {
    const parts = new Map();
    const point = new THREE.Vector3();
    let meshes = 0;
    model.group.updateMatrixWorld(true);
    model.group.traverse(mesh => {
        if (!mesh.isMesh) return;
        meshes++;
        const material = mesh.material;
        const key = [material.color.getHex(), material.emissive.getHex(), mesh.castShadow, mesh.receiveShadow].join('/');
        const positions = mesh.geometry.getAttribute('position');
        const indices = mesh.geometry.getIndex();
        if (!parts.has(key)) parts.set(key, []);
        const vertices = parts.get(key);
        for (let row = 0; row < (indices?.count ?? positions.count); row++) {
            point.fromBufferAttribute(positions, indices ? indices.getX(row) : row).applyMatrix4(mesh.matrixWorld);
            vertices.push(point.x, point.y, point.z);
        }
    });
    return { parts, meshes };
}

for (const [kind, region, w, h, style] of [
    ['house', 'hearth', 1, 1, {}],
    ['shop', 'lantern', 3, 2, {}],
    ['castle', 'hearth', 3, 2, {hall:true}],
    ['castle', 'hearth', 3, 2, {farmstead:true}],
    ['inn', 'rime', 2, 2, {}],
]) {
    const args = [kind, 2, {w,h}, region, false, {regional:true,...style}];
    const before = old.building(...args);
    const after = candidate.building(...args);
    const second = candidate.building(...args);
    assert.deepEqual([after.width,after.height,after.depth], [before.width,before.height,before.depth]);
    assert.notEqual(second.group, after.group);
    assert.equal(second.group.children[0].geometry, after.group.children[0].geometry);
    const a = surfaces(before), b = surfaces(after);
    assert(b.meshes < a.meshes);
    assert.deepEqual([...b.parts.keys()], [...a.parts.keys()]);
    for (const [material, vertices] of a.parts) {
        const actual = b.parts.get(material);
        assert.equal(actual.length, vertices.length);
        assert(vertices.every((value, index) => Math.abs(value - actual[index]) < 1e-6),
            kind + ' moved a vertex in material ' + material);
    }
    console.log(kind, region, style, a.meshes + ' → ' + b.meshes + ' meshes', 'matching triangles, materials and shadows');
}
