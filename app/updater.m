#import "updater.h"
#import <Security/Security.h>
#import <CommonCrypto/CommonDigest.h>
#include <archive.h>
#include <archive_entry.h>
#include "update_version.h"

static const unsigned long long MaxArchiveSize = 100 * 1024 * 1024;

static BOOL IsSHA256(NSString *value) {
    if (![value isKindOfClass:[NSString class]] || value.length != 64) return NO;
    NSCharacterSet *hex = [NSCharacterSet characterSetWithCharactersInString:@"0123456789abcdefABCDEF"];
    return [value rangeOfCharacterFromSet:hex.invertedSet].location == NSNotFound;
}

static NSString *FileSHA256(NSString *path) {
    NSInputStream *stream = [NSInputStream inputStreamWithFileAtPath:path];
    [stream open];
    CC_SHA256_CTX context;
    CC_SHA256_Init(&context);
    uint8_t buffer[65536];
    NSInteger count;
    while ((count = [stream read:buffer maxLength:sizeof(buffer)]) > 0) {
        CC_SHA256_Update(&context, buffer, (CC_LONG)count);
    }
    [stream close];
    if (count < 0) return nil;
    unsigned char digest[CC_SHA256_DIGEST_LENGTH];
    CC_SHA256_Final(digest, &context);
    NSMutableString *result = [NSMutableString stringWithCapacity:64];
    for (int i = 0; i < CC_SHA256_DIGEST_LENGTH; i++) [result appendFormat:@"%02x", digest[i]];
    return result;
}

// Only extract entries inside cmd-kana.app, without following archive symlinks.
static BOOL ExtractApp(NSString *zip, NSString *destination) {
    struct archive *reader = archive_read_new();
    struct archive *writer = archive_write_disk_new();
    archive_read_support_format_zip(reader);
    archive_write_disk_set_options(writer, ARCHIVE_EXTRACT_PERM |
        ARCHIVE_EXTRACT_SECURE_NODOTDOT | ARCHIVE_EXTRACT_SECURE_SYMLINKS);
    BOOL succeeded = archive_read_open_filename(reader, zip.fileSystemRepresentation, 65536) == ARCHIVE_OK;
    struct archive_entry *entry;
    unsigned long long totalSize = 0;
    unsigned entryCount = 0;
    int status = ARCHIVE_FATAL;
    while (succeeded && (status = archive_read_next_header(reader, &entry)) == ARCHIVE_OK) {
        const char *rawName = archive_entry_pathname(entry);
        NSString *name = rawName ? [NSString stringWithUTF8String:rawName] : nil;
        NSArray *components = name.pathComponents;
        mode_t type = archive_entry_filetype(entry);
        la_int64_t size = archive_entry_size(entry);
        if (!name || name.isAbsolutePath || ![components.firstObject isEqualToString:@"cmd-kana.app"] ||
            [components containsObject:@".."] || archive_entry_hardlink(entry) ||
            (type != AE_IFREG && type != AE_IFDIR && type != AE_IFLNK) ||
            size < 0 || (unsigned long long)size > MaxArchiveSize * 4 ||
            ++entryCount > 10000) {
            succeeded = NO;
            break;
        }
        totalSize += (unsigned long long)size;
        if (totalSize > MaxArchiveSize * 4) {
            succeeded = NO;
            break;
        }
        NSString *path = [destination stringByAppendingPathComponent:name];
        const char *rawLink = archive_entry_symlink(entry);
        if (rawLink) {
            NSString *link = [NSString stringWithUTF8String:rawLink];
            NSString *resolved = [[path.stringByDeletingLastPathComponent
                stringByAppendingPathComponent:link ?: @""] stringByStandardizingPath];
            NSString *appRoot = [destination stringByAppendingPathComponent:@"cmd-kana.app"];
            if (!link || link.isAbsolutePath ||
                (![resolved isEqualToString:appRoot] &&
                 ![resolved hasPrefix:[appRoot stringByAppendingString:@"/"]])) {
                succeeded = NO;
                break;
            }
        }
        archive_entry_set_pathname(entry, path.fileSystemRepresentation);
        // Do not restore special permission bits from a ZIP.
        archive_entry_set_perm(entry, archive_entry_perm(entry) & 0777);
        if (archive_write_header(writer, entry) != ARCHIVE_OK) {
            succeeded = NO;
            break;
        }
        const void *buffer;
        size_t length;
        la_int64_t offset;
        int readStatus;
        while ((readStatus = archive_read_data_block(reader, &buffer, &length, &offset)) == ARCHIVE_OK) {
            if (offset < 0 || (unsigned long long)offset > (unsigned long long)size ||
                length > (unsigned long long)size - (unsigned long long)offset ||
                archive_write_data_block(writer, buffer, length, offset) != ARCHIVE_OK) {
                succeeded = NO;
                break;
            }
        }
        if (readStatus != ARCHIVE_EOF || archive_write_finish_entry(writer) != ARCHIVE_OK) succeeded = NO;
    }
    if (status != ARCHIVE_EOF || entryCount == 0) succeeded = NO;
    archive_read_free(reader);
    if (archive_write_close(writer) != ARCHIVE_OK) succeeded = NO;
    archive_write_free(writer);
    return succeeded;
}

static BOOL HasMatchingSignature(NSString *current, NSString *candidate) {
    SecStaticCodeRef original = NULL, replacement = NULL;
    SecRequirementRef requirement = NULL;
    BOOL valid = SecStaticCodeCreateWithPath((__bridge CFURLRef)[NSURL fileURLWithPath:current],
        kSecCSDefaultFlags, &original) == errSecSuccess &&
        SecCodeCopyDesignatedRequirement(original, kSecCSDefaultFlags, &requirement) == errSecSuccess &&
        SecStaticCodeCreateWithPath((__bridge CFURLRef)[NSURL fileURLWithPath:candidate],
            kSecCSDefaultFlags, &replacement) == errSecSuccess &&
        SecStaticCodeCheckValidity(replacement, kSecCSStrictValidate | kSecCSCheckAllArchitectures | kSecCSCheckNestedCode,
            requirement) == errSecSuccess;
    if (requirement) CFRelease(requirement);
    if (replacement) CFRelease(replacement);
    if (original) CFRelease(original);
    return valid;
}

@interface CmdKanaUpdater ()
@property(readwrite) BOOL busy;
@property(strong) NSURLSession *session;
@end

@implementation CmdKanaUpdater

- (instancetype)init {
    self = [super init];
    if (self) {
        NSURLSessionConfiguration *configuration = [NSURLSessionConfiguration ephemeralSessionConfiguration];
        configuration.HTTPCookieStorage = nil;
        configuration.URLCredentialStorage = nil;
        configuration.timeoutIntervalForResource = 180.0;
        self.session = [NSURLSession sessionWithConfiguration:configuration];
    }
    return self;
}

- (void)setStatus:(NSString *)message busy:(BOOL)busy {
    self.busy = busy;
    if (self.statusChanged) self.statusChanged(message, busy);
}

- (void)finishWithMessage:(NSString *)message detail:(NSString *)detail {
    [self setStatus:message busy:NO];
    NSAlert *alert = [[NSAlert alloc] init];
    alert.messageText = message;
    alert.informativeText = detail;
    [alert addButtonWithTitle:@"OK"];
    [NSApp activateIgnoringOtherApps:YES];
    [alert runModal];
}

- (NSMutableURLRequest *)requestForURL:(NSURL *)url {
    NSMutableURLRequest *request = [NSMutableURLRequest requestWithURL:url
        cachePolicy:NSURLRequestReloadIgnoringLocalCacheData timeoutInterval:20.0];
    [request setValue:@"cmd-kana-update-check" forHTTPHeaderField:@"User-Agent"];
    return request;
}

- (NSDictionary *)assetNamed:(NSString *)name release:(NSDictionary *)release tag:(NSString *)tag {
    id assets = release[@"assets"];
    if (![assets isKindOfClass:[NSArray class]]) return nil;
    NSString *expectedURL = [NSString stringWithFormat:
        @"https://github.com/nbr41to/cmd-kana/releases/download/%@/%@", tag, name];
    for (id asset in assets) {
        if (![asset isKindOfClass:[NSDictionary class]]) continue;
        id size = asset[@"size"];
        if ([asset[@"name"] isEqual:name] && [asset[@"state"] isEqual:@"uploaded"] &&
            [asset[@"browser_download_url"] isEqual:expectedURL] &&
            [size isKindOfClass:[NSNumber class]] && [size longLongValue] > 0 &&
            [size unsignedLongLongValue] <= MaxArchiveSize) return asset;
    }
    return nil;
}

- (void)handleReleaseData:(NSData *)data response:(NSURLResponse *)response error:(NSError *)error {
    if (error || ![response isKindOfClass:[NSHTTPURLResponse class]]) {
        [self finishWithMessage:@"更新を確認できませんでした"
            detail:@"インターネット接続を確認して、もう一度お試しください。"];
        return;
    }
    NSInteger status = ((NSHTTPURLResponse *)response).statusCode;
    if (status == 404) {
        [self finishWithMessage:@"配布版はまだ公開されていません"
            detail:@"GitHub Releasesに正式版が公開されると、ここで更新を確認できます。"];
        return;
    }
    if (status != 200) {
        [self finishWithMessage:@"更新を確認できませんでした"
            detail:(status == 403 || status == 429)
                ? @"GitHubへのアクセスが制限されています。しばらくしてからお試しください。"
                : @"GitHubから更新情報を取得できませんでした。しばらくしてからお試しください。"];
        return;
    }
    id release = data ? [NSJSONSerialization JSONObjectWithData:data options:0 error:NULL] : nil;
    id tag = [release isKindOfClass:[NSDictionary class]] ? release[@"tag_name"] : nil;
    id draft = [release isKindOfClass:[NSDictionary class]] ? release[@"draft"] : nil;
    id prerelease = [release isKindOfClass:[NSDictionary class]] ? release[@"prerelease"] : nil;
    NSString *current = [[NSBundle mainBundle] objectForInfoDictionaryKey:@"CFBundleShortVersionString"];
    int comparison = [tag isKindOfClass:[NSString class]]
        ? cmd_kana_compare_versions([tag UTF8String], current.UTF8String) : CMD_KANA_VERSION_INVALID;
    if (comparison == CMD_KANA_VERSION_INVALID ||
        ![draft isKindOfClass:[NSNumber class]] || [draft boolValue] ||
        ![prerelease isKindOfClass:[NSNumber class]] || [prerelease boolValue]) {
        [self finishWithMessage:@"更新情報を読み取れませんでした"
            detail:@"正式版のバージョン情報を確認できません。しばらくしてからお試しください。"];
        return;
    }
    if (comparison <= 0) {
        [self finishWithMessage:@"利用可能な更新はありません"
            detail:[NSString stringWithFormat:@"現在のバージョンは %@、公開されている正式版は %@ です。", current, tag]];
        return;
    }
    NSString *version = [tag hasPrefix:@"v"] ? [tag substringFromIndex:1] : tag;
    NSString *baseName = [NSString stringWithFormat:@"cmd-kana-%@-macos", version];
    NSDictionary *asset = [self assetNamed:[baseName stringByAppendingString:@"-notarized.zip"] release:release tag:tag];
    if (!asset) asset = [self assetNamed:[baseName stringByAppendingString:@".zip"] release:release tag:tag];
    NSDictionary *checksum = asset ? [self assetNamed:[asset[@"name"] stringByAppendingString:@".sha256"]
        release:release tag:tag] : nil;
    id digest = asset[@"digest"];
    NSString *sha = [digest isKindOfClass:[NSString class]] && [digest hasPrefix:@"sha256:"]
        ? [digest substringFromIndex:7] : nil;
    if (!asset || (!IsSHA256(sha) && !checksum)) {
        [self finishWithMessage:@"新しい正式版の配布ファイルが準備されていません"
            detail:[NSString stringWithFormat:@"バージョン %@ のアプリZIPとSHA-256が公開されてから、もう一度お試しください。", tag]];
        return;
    }
    NSAlert *alert = [[NSAlert alloc] init];
    alert.messageText = [NSString stringWithFormat:@"バージョン %@ に更新しますか？", version];
    alert.informativeText = [NSString stringWithFormat:
        @"現在のバージョンは %@ です。ダウンロードと検証が終わると、アプリを入れ替えて再起動します。", current];
    [alert addButtonWithTitle:@"ダウンロードして更新"];
    [alert addButtonWithTitle:@"あとで"];
    [NSApp activateIgnoringOtherApps:YES];
    if ([alert runModal] != NSAlertFirstButtonReturn) {
        [self setStatus:@"更新をキャンセルしました" busy:NO];
        return;
    }
    if (IsSHA256(sha)) {
        [self downloadAsset:asset sha256:sha.lowercaseString version:version];
    } else {
        [self downloadChecksum:checksum asset:asset version:version];
    }
}

- (void)checkForUpdates {
    if (self.busy) return;
    [self setStatus:@"GitHubで更新を確認中…" busy:YES];
    NSURL *url = [NSURL URLWithString:@"https://api.github.com/repos/nbr41to/cmd-kana/releases/latest"];
    NSMutableURLRequest *request = [self requestForURL:url];
    [request setValue:@"application/vnd.github+json" forHTTPHeaderField:@"Accept"];
    [request setValue:@"2022-11-28" forHTTPHeaderField:@"X-GitHub-Api-Version"];
    [[self.session dataTaskWithRequest:request completionHandler:
        ^(NSData *data, NSURLResponse *response, NSError *error) {
            dispatch_async(dispatch_get_main_queue(), ^{
                [self handleReleaseData:data response:response error:error];
            });
        }] resume];
}

- (void)downloadChecksum:(NSDictionary *)checksum asset:(NSDictionary *)asset version:(NSString *)version {
    [self setStatus:@"更新ファイルの情報を取得中…" busy:YES];
    NSURLRequest *request = [self requestForURL:[NSURL URLWithString:checksum[@"browser_download_url"]]];
    [[self.session dataTaskWithRequest:request completionHandler:
        ^(NSData *data, NSURLResponse *response, NSError *error) {
            NSString *text = data && data.length <= 4096 ? [[NSString alloc] initWithData:data encoding:NSASCIIStringEncoding] : nil;
            NSString *sha = [text componentsSeparatedByCharactersInSet:NSCharacterSet.whitespaceAndNewlineCharacterSet].firstObject;
            BOOL valid = !error && [response isKindOfClass:[NSHTTPURLResponse class]] &&
                ((NSHTTPURLResponse *)response).statusCode == 200 && IsSHA256(sha);
            dispatch_async(dispatch_get_main_queue(), ^{
                if (!valid) {
                    [self finishWithMessage:@"更新ファイルの情報を取得できませんでした"
                        detail:@"SHA-256を確認できないため、更新を中止しました。現在のアプリはそのまま利用できます。"];
                } else {
                    [self downloadAsset:asset sha256:sha.lowercaseString version:version];
                }
            });
        }] resume];
}

- (void)downloadAsset:(NSDictionary *)asset sha256:(NSString *)sha version:(NSString *)version {
    NSFileManager *files = NSFileManager.defaultManager;
    NSString *target = [NSBundle.mainBundle.bundleURL URLByResolvingSymlinksInPath].path;
    NSString *parent = target.stringByDeletingLastPathComponent;
    if (![target.pathExtension isEqualToString:@"app"] || ![files isWritableFileAtPath:parent]) {
        [self finishWithMessage:@"アプリを入れ替えられません"
            detail:@"書き込み可能なアプリケーションフォルダへcmd-kana.appを移動してから、もう一度お試しください。"];
        return;
    }
    NSString *token = NSUUID.UUID.UUIDString;
    NSString *workspace = [[NSTemporaryDirectory() stringByAppendingPathComponent:
        [@"cmd-kana-update-" stringByAppendingString:token]] stringByResolvingSymlinksInPath];
    if (![files createDirectoryAtPath:workspace withIntermediateDirectories:NO
        attributes:@{NSFilePosixPermissions: @0700} error:NULL]) {
        [self finishWithMessage:@"更新の準備ができませんでした" detail:@"一時フォルダを作成できません。空き容量を確認してください。"];
        return;
    }
    [self setStatus:@"最新版をダウンロード中…" busy:YES];
    NSURLRequest *request = [self requestForURL:[NSURL URLWithString:asset[@"browser_download_url"]]];
    [[self.session downloadTaskWithRequest:request completionHandler:
        ^(NSURL *location, NSURLResponse *response, NSError *error) {
            NSString *zip = [workspace stringByAppendingPathComponent:@"update.zip"];
            unsigned long long size = location ? [[files attributesOfItemAtPath:location.path error:NULL] fileSize] : 0;
            BOOL downloaded = !error && location && [response isKindOfClass:[NSHTTPURLResponse class]] &&
                ((NSHTTPURLResponse *)response).statusCode == 200 && size > 0 && size <= MaxArchiveSize &&
                size == [asset[@"size"] unsignedLongLongValue] &&
                [files moveItemAtURL:location toURL:[NSURL fileURLWithPath:zip] error:NULL];
            if (!downloaded) {
                [files removeItemAtPath:workspace error:NULL];
                dispatch_async(dispatch_get_main_queue(), ^{
                    [self finishWithMessage:@"ダウンロードに失敗しました"
                        detail:@"接続と空き容量を確認して、もう一度お試しください。現在のアプリは変更していません。"];
                });
                return;
            }
            dispatch_async(dispatch_get_main_queue(), ^{ [self setStatus:@"更新ファイルを検証中…" busy:YES]; });
            [self prepareArchive:zip sha256:sha version:version target:target workspace:workspace token:token];
        }] resume];
}

- (void)prepareArchive:(NSString *)zip sha256:(NSString *)sha version:(NSString *)version
               target:(NSString *)target workspace:(NSString *)workspace token:(NSString *)token {
    NSFileManager *files = NSFileManager.defaultManager;
    NSString *extract = [workspace stringByAppendingPathComponent:@"extracted"];
    NSString *candidate = [extract stringByAppendingPathComponent:@"cmd-kana.app"];
    NSString *parent = target.stringByDeletingLastPathComponent;
    NSString *staged = [parent stringByAppendingPathComponent:
        [NSString stringWithFormat:@".cmd-kana-update-%@.app", token]];
    NSString *backup = [parent stringByAppendingPathComponent:
        [NSString stringWithFormat:@".cmd-kana-backup-%@.app", token]];
    NSString *helper = [workspace stringByAppendingPathComponent:@"cmd-kana-update-helper"];
    NSString *failure = nil;
    if (![[FileSHA256(zip) lowercaseString] isEqualToString:sha]) {
        failure = @"ダウンロードしたファイルのSHA-256が一致しません。";
    } else if (![files createDirectoryAtPath:extract withIntermediateDirectories:NO
        attributes:@{NSFilePosixPermissions: @0700} error:NULL] || !ExtractApp(zip, extract)) {
        failure = @"更新ファイルを安全に展開できませんでした。";
    } else {
        NSDictionary *info = [NSDictionary dictionaryWithContentsOfFile:
            [candidate stringByAppendingPathComponent:@"Contents/Info.plist"]];
        NSString *executable = [candidate stringByAppendingPathComponent:@"Contents/MacOS/cmd-kana"];
        if (![info[@"CFBundleIdentifier"] isEqual:NSBundle.mainBundle.bundleIdentifier] ||
            ![info[@"CFBundleShortVersionString"] isEqual:version] ||
            ![info[@"CFBundleExecutable"] isEqual:@"cmd-kana"] ||
            ![files isExecutableFileAtPath:executable]) {
            failure = @"更新先のアプリまたはバージョンが一致しません。";
        } else if (!HasMatchingSignature(target, candidate)) {
            failure = @"アプリの署名が無効か、現在のアプリと署名の条件が一致しません。";
        } else {
            NSTask *architecture = [[NSTask alloc] init];
            architecture.executableURL = [NSURL fileURLWithPath:@"/usr/bin/lipo"];
            architecture.arguments = @[@"-verify_arch", @"arm64", executable];
            architecture.standardOutput = NSFileHandle.fileHandleWithNullDevice;
            architecture.standardError = NSFileHandle.fileHandleWithNullDevice;
            BOOL launched = [architecture launchAndReturnError:NULL];
            if (launched) [architecture waitUntilExit];
            if (!launched || architecture.terminationStatus != 0) {
                failure = @"このMacに対応する実行ファイルを確認できませんでした。";
            } else if (![files copyItemAtPath:candidate toPath:staged error:NULL] ||
                ![files copyItemAtPath:NSBundle.mainBundle.executablePath toPath:helper error:NULL] ||
                ![files setAttributes:@{NSFilePosixPermissions: @0700} ofItemAtPath:helper error:NULL]) {
                failure = @"アプリの入れ替えを準備できませんでした。空き容量と書き込み権限を確認してください。";
            }
        }
    }
    dispatch_async(dispatch_get_main_queue(), ^{
        if (failure) {
            [files removeItemAtPath:staged error:NULL];
            [files removeItemAtPath:workspace error:NULL];
            [self finishWithMessage:@"更新を中止しました" detail:failure];
            return;
        }
        [self setStatus:@"アプリを入れ替えて再起動します…" busy:YES];
        NSTask *installer = [[NSTask alloc] init];
        installer.executableURL = [NSURL fileURLWithPath:helper];
        installer.arguments = @[@"--install-update",
            [NSString stringWithFormat:@"%d", NSProcessInfo.processInfo.processIdentifier],
            target, staged, backup, workspace];
        installer.standardInput = NSFileHandle.fileHandleWithNullDevice;
        installer.standardOutput = NSFileHandle.fileHandleWithNullDevice;
        installer.standardError = NSFileHandle.fileHandleWithNullDevice;
        if (![installer launchAndReturnError:NULL]) {
            [files removeItemAtPath:staged error:NULL];
            [files removeItemAtPath:workspace error:NULL];
            [self finishWithMessage:@"更新を開始できませんでした" detail:@"現在のアプリは変更していません。もう一度お試しください。"];
            return;
        }
        [NSApp terminate:nil];
    });
}

@end

void show_update_install_error(const char *message) {
    @autoreleasepool {
        [NSApplication sharedApplication];
        [NSApp setActivationPolicy:NSApplicationActivationPolicyAccessory];
        [NSApp activateIgnoringOtherApps:YES];
        NSAlert *alert = [[NSAlert alloc] init];
        alert.messageText = @"cmd-kanaを更新できませんでした";
        alert.informativeText = [NSString stringWithUTF8String:message] ?: @"もう一度お試しください。";
        [alert addButtonWithTitle:@"OK"];
        [alert runModal];
    }
}
