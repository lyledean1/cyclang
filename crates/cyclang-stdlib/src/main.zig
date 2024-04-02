const std = @import("std");

fn exportBuiltinFn(comptime func: anytype, comptime func_name: []const u8) void {
    @export(func, .{ .name = func_name, .linkage = .Strong });
}

comptime {
    exportBuiltinFn(boolToStrZig, "boolToStrExport");
}

export fn boolToStrZig(value: bool) void {
    if (value) {
        std.debug.print("this has been called from zig {s}\n", .{"true"});
    } else {
        std.debug.print("this has been called from zig {s}\n", .{"false"});
    }
}
