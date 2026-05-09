# Initial considerations

This firmware update project considers the target system (so the device whose firmware will be updated) to be perfectly secure, and to be capable of holding the keys in a matter which will not be reachable by the potential attacker. It is also assumed the firmware update will not be modified by external tasks after it had been downloaded. Unless these conditions are met, the system *will* be insecure.

Unless specified otherwise, all integers shall be stored as little endian.

The target device will host a copy of the "first stage updater", whose job it will be to verify the firmware update file and run the "second stage updater", which will be part of the firmware update file.

The firmware update file will consist of the encrypted firmware blob, and the encrypted second stage updater.

The key to the second stage updater shall be stored in the target device's filesystem, whereas the key to the firmware blob shall be stored within the second stage updater itself.

For the purpose of this demonstration, the second stage updater shall be a shell script, whose job it will be to decrypt the file and show its contents, and the update contents shall be represented by a single encrypted file with the contents of `FIRMWARE UPDATE`.

The firmware update server shall be written in Java, to make it simple to write new functionality later on. The first stage updater's implementation language will be rust, so as to make it simple to run on bare embedded devices without any underlying operating system, making the propsed OTA solution universal.

No AI shall be used in writing the code of any software related to the firmware updater. It shall only be used to detect potential vulnerabilities.

# Running the firmware update file

## The flow of operations
After the file will have been downloaded, it will be provided to the updater.

The updater will verify whether or not the file is valid, if it's signed and if its signature is known by the updater.

Following that, the file will be extracted, and the update script / binary built into the update file will be decrypted by the firmware update key, and later executed.

The sections following the second stage updater shall not be decrypted by the updater itself, and instead they should act as assets for the second stage updater, which shall decrypt them by itself.

## The outline of the firmware update file.

### Synopsis

The firmware update file shall be a custom-made archive. Existing archive files all have large attack surfaces, which makes them unsuitable for storing firmware updates.

The file will consist of three parts:

- Main header
- Sections data
- Actual contents

### Main header

The main header for now consists of the following fields (notice the section being aligned to 16 byte boundary):

- Magic number: `UPXD0001` - 8 bytes
- SHA512SUM of the (sections data + actual contents) - 64 bytes
- Key ID - 8 bytes
- Signature (RSA2048) - 256 bytes

The `Key ID` will let the updater select the correct firmware decryption key, as well as the signature verification key. It being an integer also reduces the attack surface - strings defining the pathname can get concatenated incorrectly, causing potential issues.

This layout means that the rest of the file can be treated as a "immutable blob" secured by the signature.

### Sections data

The sections data will be a list of structures, each having the following format:

- Section number - 4 bytes (UNIQUE! Non-zero)
- 4 bytes padding (Align next to 8b - has to be all zeros)
- Section size - (Has to be aligned to 16 bytes, in case format gets used by embedded devices and not a linux box) - 8 bytes
- Section SHA256SUM - 32 bytes

The sections data segment will be terminated by a struct with all fields set to \0.

Each of these structs represent a single blob of data. The section number `0x00000001` shall represent the second stage updater.

### Actual contents

Just the raw data concatenated together.
