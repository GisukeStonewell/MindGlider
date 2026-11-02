"""Verify that PE icon resources match the editable ICO and runtime PNG (stdlib only)."""
from pathlib import Path
import struct
import sys

exe = Path(sys.argv[1] if len(sys.argv) > 1 else "target/release/mindglider.exe")
data = exe.read_bytes()
pe = struct.unpack_from("<I", data, 0x3C)[0]
assert data[pe:pe+4] == b"PE\0\0"
count = struct.unpack_from("<H", data, pe+6)[0]
optional_size = struct.unpack_from("<H", data, pe+20)[0]
optional = pe+24
magic = struct.unpack_from("<H", data, optional)[0]
directories = optional+(112 if magic == 0x20B else 96)
resource_rva = struct.unpack_from("<I", data, directories+16)[0]
sections = []
for index in range(count):
    start = optional+optional_size+index*40
    virtual_size, address, size, pointer = struct.unpack_from("<IIII", data, start+8)
    sections.append((address, max(virtual_size, size), pointer))

def offset(rva):
    for address, size, pointer in sections:
        if address <= rva < address+size:
            return pointer+rva-address
    raise AssertionError("RVA is outside PE sections")

root = offset(resource_rva)

def entries(directory):
    named, ids = struct.unpack_from("<HH", data, directory+12)
    result = {}
    for index in range(named+ids):
        name, target = struct.unpack_from("<II", data, directory+16+index*8)
        if not name & 0x80000000:
            result[name] = root+(target & 0x7FFFFFFF)
    return result

def resource(kind, name):
    directory = entries(root)[kind]
    languages = entries(directory)[name]
    entry = next(iter(entries(languages).values()))
    rva, size = struct.unpack_from("<II", data, entry)
    start = offset(rva)
    return data[start:start+size]

ico = Path("assets/icons/mindglider.ico").read_bytes()
reserved, kind, count = struct.unpack_from("<HHH", ico)
assert (reserved, kind, count) == (0, 1, 7)
frames = {}
for index in range(count):
    width, height, _, _, _, bits, size, start = struct.unpack_from("<BBBBHHII", ico, 6+index*16)
    assert bits == 32
    frames[(width or 256, height or 256)] = ico[start:start+size]
group = resource(14, 1)
assert struct.unpack_from("<HHH", group) == (0, 1, 7)
for index in range(7):
    width, height, _, _, _, bits, size, name = struct.unpack_from("<BBBBHHIH", group, 6+index*14)
    image = resource(3, name)
    assert bits == 32 and len(image) == size
    assert image == frames[(width or 256, height or 256)]
assert frames[(256, 256)] == Path("assets/icons/mindglider.png").read_bytes()
assert resource(16, 1)
print("EXE: all 7 icon sizes match the ICO; runtime PNG matches; version resource present.")
