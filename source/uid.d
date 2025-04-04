module uid;

// FROM SILY PROJECT: https://github.com/al1-ce/sily
import std.format;

private ulong _inc = 0;
private ulong _seed = 0;
private const uint _uintMin = 1_000_000_000u;
private const ulong _ulongMin = 1_000_000_000_000_000_000_0u;
private const ulong _uidMask = 0x7FFFFFFFF;

uint generateUID() {
    uint _out = 0;
    ulong _tmp = _uintMin + _seed + (_inc * 0x00_10_42_1F);
    // 11_22_33_44 -> 22_44_11_33
    _out += (_tmp & 0x00_00_00_FF) << 16;
    _out += (_tmp & 0x00_00_FF_00) >> 8;
    _out += (_tmp & 0x00_FF_00_00) << 8;
    _out += (_tmp & 0xFF_00_00_00) >> 16;
    if (_out < _uintMin) _out += _uintMin;
    ++_inc;
    return _out;
}

/// Returns hex string of 32 bit uid (8 letters)
string genStringUID() {
    return format("%x", generateUID());
}