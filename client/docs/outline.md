# Initial considerations

This desktop app updater project considers the target system (so the device on which the application will be installed) to be secure, and to be capable of holding the public keys in a matter which will not be editable by the potential attacker.

Unless specified otherwise, all integers shall be stored as little endian.

The target will host a copy of the "first stage updater", whose job it will be to download and verify the update file and run the "second stage updater" or "installer", which will be part of the update file, and which will actually replace the application's files on the target.

For the purpose of this demonstration, the application will be a single file called `app`, and the second stage updater - a shell file which will replace the `app` file when ran.

The update server shall be written in Java, to make it simple to write new functionality later on. The first stage updater's implementation language will be rust, so as to make it simple, secure and cross-platform.

No AI shall be used in writing the code of any software related to the updater. It shall only be used to detect potential vulnerabilities.

# Update server

The server's job is to do two things:

- Check if a given target has an update available
- Provide the update installer

## Cohorts

In order to prevent a bad update from affecting a large amount of users, the updates should be rolled out in waves. Users which should receive the update at the same time all should belong to one cohort.

So as to keep things simple during the presentation, the function mapping the installation's random serial number to a given cohort is as follows:

```
cohort = javaHashCode(serial_number) % 16
```

This should split the userbase into 16 cohorts. There will be 16 groups of users, each getting the update file at a different time. In case of a bad update, only a group of users should be affected, instead of everyone.

## Serial numbers

The serial numbers should encode the application type, as well as the "real" serial number.
The format is as follows: `APPID-serial` where `APPID` is a application identifier (5 chars), akin to a model number - what we are updating, and `serial` being an UUID.

## Transport layer

The update server shall provide a HTTPS port,
and the stage one updater shall use certificate pinning to make it harder to swap servers.

# Running the update installer

## The flow of operations
After the file will have been downloaded, it will be verified by the updater and executed.

The updater will verify whether or not the file is valid, if it's signed and if its signature is known by the updater.

Following that, the file will be extracted, and the update script / binary built into the update file will executed.

## The outline of the update file.

### Synopsis

The update file shall be a custom-made archive. Existing archive files all have large attack surfaces, which makes them unsuitable for storing updates.

The file will consist of three parts:

- Main header
- Additional metadata
- Second stage updater

### Main header

The main header for now consists of the following fields (notice the section being aligned to 16 byte boundary):

- Magic number: `UPXD0001` - 8 bytes
- Key ID - 8 bytes
- SHA512SUM of the (additional metadata + second stage updater) - 64 bytes
- Signature (ML-DSA 87) of the SHASUM - 4627 bytes

The `Key ID` will let the updater select the correct signature verification key. It being an integer also reduces the attack surface - strings defining the pathname can get concatenated incorrectly, causing potential issues.

This layout means that the rest of the file can be treated as a "immutable blob" secured by the signature.

### Additional metadata

The additional metadata section is meant to describe additional data which may be used to make sure the update should be peformed, and that an illegal operation, such as a downgrade, cannot happen.

Structure:

- Semver (2 bytes for each: major, minor, patch, alpha) - 8 bytes in total.
- Length (the cumulative length of all subsequent sections) - 8 bytes.

### Second stage updater / installer

Just the raw installer data
