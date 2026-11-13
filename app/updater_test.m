// Include the implementation so its archive/hash/signature helpers are tested directly.
#import "updater.m"
#include <assert.h>

static void WriteZip(NSString *path, NSString *name, NSString *link) {
    struct archive *writer = archive_write_new();
    assert(archive_write_set_format_zip(writer) == ARCHIVE_OK);
    assert(archive_write_open_filename(writer, path.fileSystemRepresentation) == ARCHIVE_OK);
    struct archive_entry *entry = archive_entry_new();
    archive_entry_set_pathname(entry, name.UTF8String);
    archive_entry_set_perm(entry, 0755);
    archive_entry_set_filetype(entry, link ? AE_IFLNK : AE_IFREG);
    if (link) archive_entry_set_symlink(entry, link.UTF8String);
    archive_entry_set_size(entry, link ? 0 : 3);
    assert(archive_write_header(writer, entry) == ARCHIVE_OK);
    if (!link) assert(archive_write_data(writer, "abc", 3) == 3);
    archive_entry_free(entry);
    assert(archive_write_close(writer) == ARCHIVE_OK);
    archive_write_free(writer);
}

static NSString *SignedApp(NSString *root, NSString *name, NSString *identifier, NSString *version) {
    NSFileManager *files = NSFileManager.defaultManager;
    NSString *app = [root stringByAppendingPathComponent:name];
    NSString *macos = [app stringByAppendingPathComponent:@"Contents/MacOS"];
    NSString *resources = [app stringByAppendingPathComponent:@"Contents/Resources"];
    assert([files createDirectoryAtPath:macos withIntermediateDirectories:YES attributes:nil error:NULL]);
    assert([files createDirectoryAtPath:resources withIntermediateDirectories:YES attributes:nil error:NULL]);
    assert([files copyItemAtPath:@"/usr/bin/true" toPath:[macos stringByAppendingPathComponent:@"cmd-kana"] error:NULL]);
    NSDictionary *info = @{@"CFBundleIdentifier": identifier, @"CFBundleExecutable": @"cmd-kana",
        @"CFBundlePackageType": @"APPL", @"CFBundleShortVersionString": version};
    assert([info writeToFile:[app stringByAppendingPathComponent:@"Contents/Info.plist"] atomically:YES]);
    assert([@"original" writeToFile:[resources stringByAppendingPathComponent:@"test"]
        atomically:YES encoding:NSUTF8StringEncoding error:NULL]);
    NSTask *sign = [[NSTask alloc] init];
    sign.executableURL = [NSURL fileURLWithPath:@"/usr/bin/codesign"];
    sign.arguments = @[@"--force", @"--sign", @"-", @"--requirements",
        [NSString stringWithFormat:@"=designated => identifier \"%@\"", identifier], app];
    assert([sign launchAndReturnError:NULL]);
    [sign waitUntilExit];
    assert(sign.terminationStatus == 0);
    return app;
}

int main(void) {
    @autoreleasepool {
        NSFileManager *files = NSFileManager.defaultManager;
        NSString *root = [[NSTemporaryDirectory() stringByAppendingPathComponent:NSUUID.UUID.UUIDString]
            stringByResolvingSymlinksInPath];
        assert([files createDirectoryAtPath:root withIntermediateDirectories:NO attributes:nil error:NULL]);
        NSString *input = [root stringByAppendingPathComponent:@"input"];
        assert([@"abc" writeToFile:input atomically:YES encoding:NSUTF8StringEncoding error:NULL]);
        assert([FileSHA256(input) isEqualToString:@"ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"]);
        assert(FileSHA256([root stringByAppendingPathComponent:@"missing"]) == nil);
        assert(IsSHA256(FileSHA256(input)));
        assert(!IsSHA256(@"not a checksum"));

        NSString *zip = [root stringByAppendingPathComponent:@"update.zip"];
        NSString *destination = [root stringByAppendingPathComponent:@"extracted"];
        assert([files createDirectoryAtPath:destination withIntermediateDirectories:NO attributes:nil error:NULL]);
        WriteZip(zip, @"cmd-kana.app/Contents/test", nil);
        assert(ExtractApp(zip, destination));
        assert([[NSString stringWithContentsOfFile:[destination stringByAppendingPathComponent:@"cmd-kana.app/Contents/test"]
            encoding:NSUTF8StringEncoding error:NULL] isEqualToString:@"abc"]);
        WriteZip(zip, @"cmd-kana.app/../../outside", nil);
        assert(!ExtractApp(zip, destination));
        WriteZip(zip, [root stringByAppendingPathComponent:@"outside"], nil);
        assert(!ExtractApp(zip, destination));
        WriteZip(zip, @"cmd-kana.app/escape", @"../../outside");
        assert(!ExtractApp(zip, destination));
        assert(![files fileExistsAtPath:[root stringByAppendingPathComponent:@"outside"]]);

        NSString *original = SignedApp(root, @"original.app", @"local.cmd-kana", @"0.9.3");
        NSString *updated = SignedApp(root, @"updated.app", @"local.cmd-kana", @"0.10.0");
        NSString *other = SignedApp(root, @"other.app", @"example.other", @"0.10.0");
        assert(HasMatchingSignature(original, updated));
        assert(!HasMatchingSignature(original, other));
        assert([@"tampered" writeToFile:[updated stringByAppendingPathComponent:@"Contents/Resources/test"]
            atomically:YES encoding:NSUTF8StringEncoding error:NULL]);
        assert(!HasMatchingSignature(original, updated));
        assert([files removeItemAtPath:root error:NULL]);
    }
    return 0;
}
