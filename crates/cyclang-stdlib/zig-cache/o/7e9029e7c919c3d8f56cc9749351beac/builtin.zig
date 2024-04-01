const std = @import("std");
/// Zig version. When writing code that supports multiple versions of Zig, prefer
/// feature detection (i.e. with `@hasDecl` or `@hasField`) over version checks.
pub const zig_version = std.SemanticVersion.parse(zig_version_string) catch unreachable;
pub const zig_version_string = "0.11.0";
pub const zig_backend = std.builtin.CompilerBackend.stage2_llvm;

pub const output_mode = std.builtin.OutputMode.Obj;
pub const link_mode = std.builtin.LinkMode.Static;
pub const is_test = false;
pub const single_threaded = false;
pub const abi = std.Target.Abi.none;
pub const cpu: std.Target.Cpu = .{
    .arch = .aarch64,
    .model = &std.Target.aarch64.cpu.generic,
    .features = std.Target.aarch64.featureSet(&[_]std.Target.aarch64.Feature{
        .enable_select_opt,
        .ete,
        .fp_armv8,
        .fuse_adrp_add,
        .fuse_aes,
        .neon,
        .trbe,
        .use_postra_scheduler,
    }),
};
pub const os = std.Target.Os{
    .tag = .linux,
    .version_range = .{ .linux = .{
        .range = .{
            .min = .{
                .major = 3,
                .minor = 16,
                .patch = 0,
            },
            .max = .{
                .major = 5,
                .minor = 10,
                .patch = 81,
            },
        },
        .glibc = .{
            .major = 2,
            .minor = 19,
            .patch = 0,
        },
    }},
};
pub const target = std.Target{
    .cpu = cpu,
    .os = os,
    .abi = abi,
    .ofmt = object_format,
};
pub const object_format = std.Target.ObjectFormat.elf;
pub const mode = std.builtin.Mode.Debug;
pub const link_libc = false;
pub const link_libcpp = false;
pub const have_error_return_tracing = false;
pub const valgrind_support = true;
pub const sanitize_thread = false;
pub const position_independent_code = false;
pub const position_independent_executable = false;
pub const strip_debug_info = true;
pub const code_model = std.builtin.CodeModel.default;
pub const omit_frame_pointer = false;
