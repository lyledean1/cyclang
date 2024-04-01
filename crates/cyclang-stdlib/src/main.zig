const std = @import("std");
const testing = std.testing;

fn exportBuiltinFn(comptime func: anytype, comptime func_name: []const u8) void {
    @export(func, .{ .name = "cyclang_stdlib." ++ func_name, .linkage = .Strong });
}

comptime {
    exportBuiltinFn(boolToStrZig, "boolToStr");
}

export fn boolToStrZig(value: bool) void {
    if (value) {
        std.debug.print("this is zig {s}", .{"true"});
    } else {
        std.debug.print("this is zig {s}", .{"false"});
    }
}

export fn add(a: i32, b: i32) i32 {
    return a + b;
}

test "basic add functionality" {
    try testing.expect(add(3, 7) == 10);
}
