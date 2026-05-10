u8 magic[8]@$ [[color("ff0000")]];
u64 keyid@$ [[color("0000ff")]];

u8 shasum[64]@$ [[color("00ff00")]];
u8 signature[4627]@$ [[color("ffff00")]];

struct Semver {
    u16 major, minor, patch, alpha;
};

Semver semver@$ [[color("ff0000")]];
u64 fileSize@$ [[color("0000ff")]];


