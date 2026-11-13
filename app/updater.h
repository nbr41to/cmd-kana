#import <Cocoa/Cocoa.h>

@interface CmdKanaUpdater : NSObject
@property(readonly) BOOL busy;
@property(copy) void (^statusChanged)(NSString *message, BOOL busy);
- (void)checkForUpdates;
@end
