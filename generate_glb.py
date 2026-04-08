import struct
import json

# Создаём простую модель машины (UAZ Patriot style) в формате GLB
# GLB формат: JSON header + JSON chunk + BIN chunk

def create_vertex(x, y, z, nx, ny, nz, u, v):
    return struct.pack('<ffffffff', x, y, z, nx, ny, nz, u, v)

def create_index(i):
    return struct.pack('<I', i)

vertices = []
indices = []

# Кузов машины (упрощённая коробка)
length = 4.5
width = 2.0
height = 1.8
wheel_base = 2.7

# Позиция центра
cx, cy, cz = 0, 0.5, 0

# Вершины кузова (24 вершины для 6 граней)
# Front
v0 = create_vertex(cx - width/2, cy - height/2, cz + length/2, 0, 0, 1, 0, 0)
v1 = create_vertex(cx + width/2, cy - height/2, cz + length/2, 0, 0, 1, 1, 0)
v2 = create_vertex(cx + width/2, cy + height/2, cz + length/2, 0, 0, 1, 1, 1)
v3 = create_vertex(cx - width/2, cy + height/2, cz + length/2, 0, 0, 1, 0, 1)
vertices.extend([v0, v1, v2, v3])
indices.extend([0, 1, 2, 0, 2, 3])

# Back
v4 = create_vertex(cx + width/2, cy - height/2, cz - length/2, 0, 0, -1, 0, 0)
v5 = create_vertex(cx - width/2, cy - height/2, cz - length/2, 0, 0, -1, 1, 0)
v6 = create_vertex(cx - width/2, cy + height/2, cz - length/2, 0, 0, -1, 1, 1)
v7 = create_vertex(cx + width/2, cy + height/2, cz - length/2, 0, 0, -1, 0, 1)
vertices.extend([v4, v5, v6, v7])
indices.extend([4, 5, 6, 4, 6, 7])

# Top
v8 = create_vertex(cx - width/2, cy + height/2, cz + length/2, 0, 1, 0, 0, 0)
v9 = create_vertex(cx + width/2, cy + height/2, cz + length/2, 0, 1, 0, 1, 0)
v10 = create_vertex(cx + width/2, cy + height/2, cz - length/2, 0, 1, 0, 1, 1)
v11 = create_vertex(cx - width/2, cy + height/2, cz - length/2, 0, 1, 0, 0, 1)
vertices.extend([v8, v9, v10, v11])
indices.extend([8, 9, 10, 8, 10, 11])

# Bottom
v12 = create_vertex(cx + width/2, cy - height/2, cz + length/2, 0, -1, 0, 0, 0)
v13 = create_vertex(cx - width/2, cy - height/2, cz + length/2, 0, -1, 0, 1, 0)
v14 = create_vertex(cx - width/2, cy - height/2, cz - length/2, 0, -1, 0, 1, 1)
v15 = create_vertex(cx + width/2, cy - height/2, cz - length/2, 0, -1, 0, 0, 1)
vertices.extend([v12, v13, v14, v15])
indices.extend([12, 13, 14, 12, 14, 15])

# Left
v16 = create_vertex(cx - width/2, cy - height/2, cz + length/2, -1, 0, 0, 0, 0)
v17 = create_vertex(cx - width/2, cy + height/2, cz + length/2, -1, 0, 0, 1, 0)
v18 = create_vertex(cx - width/2, cy + height/2, cz - length/2, -1, 0, 0, 1, 1)
v19 = create_vertex(cx - width/2, cy - height/2, cz - length/2, -1, 0, 0, 0, 1)
vertices.extend([v16, v17, v18, v19])
indices.extend([16, 17, 18, 16, 18, 19])

# Right
v20 = create_vertex(cx + width/2, cy - height/2, cz - length/2, 1, 0, 0, 0, 0)
v21 = create_vertex(cx + width/2, cy + height/2, cz - length/2, 1, 0, 0, 1, 0)
v22 = create_vertex(cx + width/2, cy + height/2, cz + length/2, 1, 0, 0, 1, 1)
v23 = create_vertex(cx + width/2, cy - height/2, cz + length/2, 1, 0, 0, 0, 1)
vertices.extend([v20, v21, v22, v23])
indices.extend([20, 21, 22, 20, 22, 23])

bin_data = b''.join(vertices) + b''.join([create_index(i) for i in indices])

# GLTF JSON
gltf = {
    "asset": {"version": "2.0", "generator": "Python GLB Generator"},
    "scene": 0,
    "scenes": [{"nodes": [0]}],
    "nodes": [{"mesh": 0, "name": "uaz_patriot"}],
    "meshes": [{
        "primitives": [{
            "attributes": {"POSITION": 0, "NORMAL": 1, "TEXCOORD_0": 2},
            "indices": 3,
            "mode": 4
        }]
    }],
    "accessors": [
        {"bufferView": 0, "componentType": 5126, "count": 24, "type": "VEC3", "max": [1.0, 1.4, 2.25], "min": [-1.0, -0.4, -2.25]},
        {"bufferView": 0, "componentType": 5126, "count": 24, "type": "VEC3", "offset": 288},
        {"bufferView": 0, "componentType": 5126, "count": 24, "type": "VEC2", "offset": 576},
        {"bufferView": 1, "componentType": 5125, "count": 36, "type": "SCALAR"}
    ],
    "bufferViews": [
        {"buffer": 0, "byteOffset": 0, "byteLength": 24 * 32},
        {"buffer": 0, "byteOffset": 24 * 32, "byteLength": 36 * 4}
    ],
    "buffers": [{"byteLength": len(bin_data)}]
}

json_str = json.dumps(gltf, separators=(',', ':'))
# Выравниваем JSON до 4 байт
json_padding = (4 - len(json_str) % 4) % 4
json_str += ' ' * json_padding

bin_padding = (4 - len(bin_data) % 4) % 4
bin_data += b'\x00' * bin_padding

# GLB Header
glb_header = struct.pack('<III', 0x46546C67, 2, 12 + 8 + len(json_str) + 8 + len(bin_data))
# JSON Chunk
json_chunk = struct.pack('<II', len(json_str), 0x4E4F534A) + json_str.encode('utf-8')
# BIN Chunk
bin_chunk = struct.pack('<II', len(bin_data), 0x004E4942) + bin_data

glb_data = glb_header + json_chunk + bin_chunk

with open('assets/models/uaz_patriot.glb', 'wb') as f:
    f.write(glb_data)

print(f"Created uaz_patriot.glb ({len(glb_data)} bytes)")
